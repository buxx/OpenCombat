use oc_utils::index::AtIndex;
use std::collections::HashSet;
use std::path::Path;

use anyhow::Context;
use derive_more::{Deref, DerefMut};
use glam::Vec2;
use oc_geo::region::WorldRegionIndex;
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
pub mod vehicle;
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
        snapshot.vehicles.clear();
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
        if let Some(vehicle) = &squad.vehicle {
            vehicle
                .validate(mod_, squad)
                .context(format!("Validate vehicle of squad {}", squad.uuid))?;
        }

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
        let illustration = mod_
            .illustration_from_name(IllustrationKind::IngameSquad, &squad.illustration)
            .context(format!(
                "Use squad ingame illustration '{}'",
                squad.illustration
            ))?
            .index();

        if let Some(vehicle) = &squad.vehicle {
            Self::fill_vehicle(
                w,
                mod_,
                snapshot,
                side,
                squad,
                vehicle,
                &individuals,
                position,
                illustration,
                tiles,
            )?;
        }

        let squad = Squad::fresh(side, individuals.clone(), position);
        let squad = squad.with_illustration(illustration);
        snapshot.squads.push(squad);

        Ok(())
    }

    /// Place squad vehicle at squad `position` and embark its crew.
    #[allow(clippy::too_many_arguments)]
    fn fill_vehicle(
        w: &WorldConfig,
        mod_: &Mod,
        snapshot: &mut Snapshot,
        side: Side,
        squad: &squad::Squad,
        vehicle: &vehicle::Vehicle,
        individuals: &[IndividualIndex],
        position: WorldVec2,
        illustration: oc_mod::illustration::IllustrationIndex,
        tiles: &[Tile],
    ) -> Result<(), anyhow::Error> {
        let type_ = mod_
            .vehicle_from_name(&vehicle.name)
            .context(format!("Use vehicle '{}'", vehicle.name))?
            .index();
        let tile = WorldTileIndex::from_(position, w);
        let position = position.extend(tiles.at(tile).z_pixels(w));
        let vehicle_index = oc_vehicle::VehicleIndex(snapshot.vehicles.len() as u64);

        for member in &vehicle.crew {
            let i = squad
                .individuals
                .iter()
                .position(|i| i.uuid == member.individual)
                .and_then(|i| individuals.get(i))
                .context(format!("Find crew individual {}", member.individual))?;
            let individual = snapshot.individuals.at_mut(*i);
            individual.embarked = Some((vehicle_index, member.role));
            individual.position = position;
            individual.tile = tile;
            individual.region = WorldRegionIndex::from_(tile, w);
        }

        snapshot.vehicles.push(oc_vehicle::Vehicle::new(
            side,
            type_,
            illustration,
            position,
            vec![],
            oc_utils::d2::Direction::NORTH,
            Some(oc_utils::d2::Direction::NORTH),
        ));

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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use oc_mod::{nature::NatureIndex, vehicle::CrewRole};
    use oc_root::geo::WorldVec3;

    use super::*;
    use crate::deployment::vehicle::{CrewMember, Vehicle};

    fn mod_() -> Mod {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../mods/tests1");
        Mod::load(&path, None).unwrap()
    }

    #[test]
    fn fill_vehicle_embarks_crew() {
        // Given
        let mod_ = mod_();
        let w = WorldConfig::new(10, 10);
        let tiles: Vec<Tile> = (0..w.tiles_count())
            .map(|i| {
                let traversability = mod_.nature(NatureIndex(0)).traversability.clone();
                Tile::new(WorldTileIndex(i), NatureIndex(0), 0, traversability)
            })
            .collect();
        let mut squad = squad::Squad::new();
        squad.individuals = (0..3).map(|_| individual::Individual::new()).collect();
        let mut vehicle = Vehicle::new("Tank1".to_string());
        vehicle.crew = vec![
            CrewMember {
                role: CrewRole::Driver,
                individual: squad.individuals[2].uuid,
            },
            CrewMember {
                role: CrewRole::Commander,
                individual: squad.individuals[0].uuid,
            },
        ];
        let mut snapshot = Snapshot::empty(w.clone());
        // An individual of a previous squad, then squad members
        let mut individuals = vec![];
        for i in 0..4 {
            let position = WorldVec3::new(i as f32, i as f32, 0.);
            snapshot
                .individuals
                .push(Individual::fresh(&w, Side::A, position));
            if i > 0 {
                individuals.push(IndividualIndex(i));
            }
        }
        let position = WorldVec2::new(22., 33.);

        // When
        Deployments::fill_vehicle(
            &w,
            &mod_,
            &mut snapshot,
            Side::A,
            &squad,
            &vehicle,
            &individuals,
            position,
            oc_mod::illustration::IllustrationIndex(0),
            &tiles,
        )
        .unwrap();

        // Then
        let index = oc_vehicle::VehicleIndex(0);
        assert_eq!(snapshot.vehicles.len(), 1);
        assert_eq!(
            snapshot.vehicles[0].type_,
            mod_.vehicle_from_name("Tank1").unwrap().index()
        );
        assert_eq!(snapshot.vehicles[0].position, WorldVec3::new(22., 33., 0.));
        assert_eq!(snapshot.individuals[0].embarked, None);
        assert_eq!(
            snapshot.individuals[1].embarked,
            Some((index, CrewRole::Commander))
        );
        assert_eq!(snapshot.individuals[2].embarked, None);
        assert_eq!(
            snapshot.individuals[3].embarked,
            Some((index, CrewRole::Driver))
        );
        assert_eq!(
            snapshot.individuals[1].position,
            WorldVec3::new(22., 33., 0.)
        );
        assert_eq!(
            snapshot.individuals[3].position,
            WorldVec3::new(22., 33., 0.)
        );
        assert_eq!(
            snapshot.individuals[3].tile,
            WorldTileIndex::from_(WorldVec2::new(22., 33.), &w)
        );
    }
}
