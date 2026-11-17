use oc_utils::index::AtIndex;
use std::collections::HashSet;
use std::path::Path;

use anyhow::Context;
use derive_more::{Deref, DerefMut};
use glam::Vec2;
use oc_geo::tile::{TileXy, WorldTileIndex};
use oc_individual::Xp;
use oc_individual::{
    Individual, IndividualIndex, Weapon,
    squad::{Squad, SquadFormation},
};
use oc_mod::{Mod, illustration::IllustrationKind};
use oc_root::{
    WcfgFrom, WorldConfig,
    geo::WorldVec2,
    material::MaterialKind,
    side::{BySide, Side},
    y::V,
};
use oc_utils::{
    d2::{Angle, Xy},
    polygon::Polygon,
};
use oc_world::{snapshot::Snapshot, spawn::SpawnZone, tile::Tile};
use serde::{Deserialize, Serialize};

use crate::deployment::place::Placer;

pub mod individual;
pub mod magazine;
pub mod place;
pub mod squad;
pub mod weapon;

/// Deployment of each side
#[derive(Debug, Deserialize, Serialize, Default, Deref, DerefMut)]
#[serde(transparent)]
pub struct Deployments(pub BySide<Deployment>);

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Deployment {
    pub squads: Vec<squad::Squad>,
    #[serde(default)]
    pub spawns: Vec<SpawnZone>,
}

impl Deployment {
    pub fn from_file(path: &Path) -> Result<Self, anyhow::Error> {
        Ok(oc_root::format::read_yaml(path)?)
    }
}

impl Deployments {
    pub fn from_files(side_a: &Path, side_b: &Path) -> Result<Self, anyhow::Error> {
        Ok(Self(BySide::new(
            Deployment::from_file(side_a)?,
            Deployment::from_file(side_b)?,
        )))
    }

    pub fn with_spawns(mut self, side_a: Vec<SpawnZone>, side_b: Vec<SpawnZone>) -> Self {
        self.a.spawns = side_a;
        self.b.spawns = side_b;
        self
    }

    pub fn mobilize(
        &self,
        w: &WorldConfig,
        mod_: &Mod,
        snapshot: &mut Snapshot,
        placer: &Placer,
        tiles: &[Tile],
    ) -> Result<(), anyhow::Error> {
        snapshot.squads.clear();
        snapshot.individuals.clear();
        snapshot.uuids.individuals.clear();
        snapshot.uuids.squads.clear();
        let mut i: u64 = 0;

        for (side, deployment) in self.iter() {
            for squad in &deployment.squads {
                self.fill_squad(w, mod_, snapshot, placer, &mut i, side, squad, tiles)?;
            }
        }

        // Orders (do it here after fills to have all individuals when compute orders)
        for (i, squad) in snapshot.squads.clone().iter().enumerate() {
            let orders = placer.orders(w, mod_, snapshot, squad);
            let squad = squad.clone().with_orders(orders);
            snapshot.squads[i] = squad;
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fill_squad(
        &self,
        w: &WorldConfig,
        mod_: &Mod,
        snapshot: &mut Snapshot,
        placer: &Placer,
        i: &mut u64,
        side: Side,
        squad: &squad::Squad,
        tiles: &[Tile],
    ) -> Result<(), anyhow::Error> {
        let mut individuals = vec![];
        let spawns = &self[side].spawns;
        let occupied: Vec<WorldVec2> = snapshot.squads.iter().map(|s| s.position).collect();
        let position = placer.place(spawns, squad.uuid, &occupied);
        let places = Self::members_positions(
            w,
            mod_,
            tiles,
            spawns,
            position,
            Angle::zero(),
            squad.individuals.len(),
        );

        for (indiv, position_) in squad.individuals.iter().zip(places) {
            let individual = self.fill_individual(w, mod_, i, side, indiv, position_, tiles)?;
            individuals.push(IndividualIndex(*i));
            snapshot.individuals.push(individual);
            snapshot.uuids.individuals.push(indiv.uuid);
            *i += 1;
        }

        snapshot.uuids.squads.push(squad.uuid);
        let squad = Squad::fresh(side, individuals.clone(), position).with_illustration(
            mod_.illustration_from_name(IllustrationKind::IngameSquad, &squad.illustration)
                .context(format!(
                    "Use squad ingame illustration '{}'",
                    squad.illustration
                ))?
                .index(),
        );
        snapshot.squads.push(squad);

        Ok(())
    }

    /// Members are placed on the best opacity tile near their formation place. Formation is a
    /// line along `angle` (0.0 for horizontal), leader at `position`.
    pub fn members_positions(
        w: &WorldConfig,
        mod_: &Mod,
        tiles: &[Tile],
        spawns: &[SpawnZone],
        position: WorldVec2,
        angle: Angle,
        count: usize,
    ) -> Vec<WorldVec2> {
        let reference = Vec2::new(position.x, position.y);
        let places = SquadFormation::Line.positions(w, V::Server, reference, angle, count);
        let mut taken: HashSet<WorldTileIndex> = HashSet::new();

        places
            .into_iter()
            .map(|slot| {
                let slot = WorldVec2::new(slot.x, slot.y);
                Self::best_tile_position(w, mod_, tiles, spawns, slot, position, &mut taken)
            })
            .collect()
    }

    // Warning: AI generated
    /// Take the free tile with the best opacity within `formation_tiles_radius_placement`
    /// tiles of `slot`, and mark it as taken. Returns the tile center.
    fn best_tile_position(
        w: &WorldConfig,
        mod_: &Mod,
        tiles: &[Tile],
        spawns: &[SpawnZone],
        place: WorldVec2,
        fallback: WorldVec2,
        taken: &mut HashSet<WorldTileIndex>,
    ) -> WorldVec2 {
        let radius = w.formation_tiles_radius_placement() as i64;
        let TileXy(Xy(cx, cy)) = TileXy::from_(place, w);
        let mut best: Option<(f32, WorldTileIndex, TileXy)> = None;

        for y in (cy as i64 - radius)..=(cy as i64 + radius) {
            for x in (cx as i64 - radius)..=(cx as i64 + radius) {
                if x < 0 || y < 0 || x >= w.world_width() as i64 || y >= w.world_height() as i64 {
                    continue;
                }
                let xy = TileXy(Xy(x as u64, y as u64));
                let index = WorldTileIndex::from_(xy, w);
                let tile = tiles.at(index);
                if taken.contains(&index) || tile.allow.deny(MaterialKind::Individual) {
                    continue;
                }
                // Members must stay in owned spawn zones
                let [px, py] = xy.point(w);
                let half = w.geo_pixels_per_tile() as f32 / 2.;
                let center = WorldVec2::new(px + half, py + half);
                if !spawns.iter().any(|s| s.contains(center)) {
                    continue;
                }

                let opacity = mod_.nature(tile.nature).opacity;
                if best.is_none_or(|(best_opacity, _, _)| opacity > best_opacity) {
                    best = Some((opacity, index, xy));
                }
            }
        }

        // No free walkable tile around in spawn zones: stay on squad position
        let Some((_, index, xy)) = best else {
            return fallback;
        };
        taken.insert(index);
        let [x, y] = xy.point(w);
        let half = w.geo_pixels_per_tile() as f32 / 2.;
        WorldVec2::new(x + half, y + half)
    }

    #[allow(clippy::too_many_arguments)]
    fn fill_individual(
        &self,
        w: &WorldConfig,
        mod_: &Mod,
        individual_index: &mut u64,
        side: Side,
        individual_: &individual::Individual,
        squad_position: WorldVec2,
        tiles: &[Tile],
    ) -> Result<Individual, anyhow::Error> {
        // FIXME BS NOW: z; and according to formation (and according to best opacity tile)
        let tile = WorldTileIndex::from_(squad_position, w);
        let tile = tiles.at(tile);
        let z = tile.z_pixels(w);
        let position = squad_position.extend(z);
        let mut individual = Individual::fresh(w, side, position)
            .with_illustration(
                mod_.illustration_from_name(
                    IllustrationKind::IngameIndividual,
                    &individual_.illustration,
                )
                .context(format!(
                    "Use individual ingame illustration '{}'",
                    individual_.illustration
                ))?
                .index(),
            )
            .with_xp(Xp(individual_.xp));

        // Magazine inserted in the weapon is no longer carried
        let mut inserted = None;
        if let Some(weapon) = &individual_.weapons.main {
            let weapon = mod_.weapon_from_name(weapon)?;
            let (filled, filled_count) =
                self.fill_weapon(mod_, individual_, weapon)
                    .context(format!(
                        "Fill weapon for individual {individual_index} and mod {}",
                        mod_.name()
                    ))?;
            inserted = filled.map(|(magazine, _)| magazine);
            individual.weapons.primary = Some(Weapon {
                i: weapon.index(),
                filled,
                filled_count,
            });
        }

        for grenade in &individual_.weapons.grenades {
            let grenade = mod_.grenade_from_name(grenade).context(format!(
                "Fill grenade for individual {individual_index} and grenade {grenade} and mod {}",
                mod_.name()
            ))?;
            individual.weapons.grenades.push(grenade.index());
        }

        for magazine in &individual_.magazines {
            let magazine = mod_.magazine_from_name(&magazine.name).context(format!(
                "Fill magazine for individual {individual_index} and magazine {} and mod {}",
                magazine.name,
                mod_.name()
            ))?;

            if inserted == Some(magazine.index()) {
                inserted = None;
                continue;
            }

            individual.magazines.push(magazine.index());
        }

        Ok(individual)
    }

    // Search from deployment individual magazines if one match with weapon compatible magazines
    fn fill_weapon(
        &self,
        mod_: &Mod,
        deployment_individual: &individual::Individual,
        weapon: &oc_mod::weapons::IndexedWeapon,
    ) -> Result<
        (
            Option<(
                oc_mod::magazine::MagazineIndex,
                oc_mod::ammunition::AmmunitionIndex,
            )>,
            u16,
        ),
        anyhow::Error,
    > {
        for magazine in &deployment_individual.magazines {
            // If it is an accepted magazine by this weapon
            if weapon.magazines().iter().any(|m| m.name() == magazine.name) {
                // Read real magazine from mod
                let magazine = mod_.magazine_from_name(&magazine.name)?;

                // This magazine must accept ammunition model which is compatible with weapon
                let magazine_ammunitions = mod_.ammunitions_from_model_names(magazine.accept());
                if let Some(ammunition) = weapon
                    .ammunitions()
                    .iter()
                    .find(|a| magazine_ammunitions.contains(a))
                {
                    return Ok((
                        Some((magazine.index(), ammunition.index())),
                        magazine.capacity(),
                    ));
                }
            }
        }

        Ok((None, 0))
    }
}
