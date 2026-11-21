use oc_utils::index::AtIndex;
use oc_utils::percent::Percent;
use std::time::{Duration, Instant};

use line_drawing::Bresenham3d;
use oc_geo::region::WorldRegionIndex;
use oc_geo::tile::{TileXy, WorldTileIndex};
use oc_individual::{IndividualIndex, Status, Update as IndividualUpdate};
use oc_mod::Mod;
use oc_mod::nature::NatureIndex;
use oc_physics::fx;
use oc_root::alter::Alter;
use oc_root::explosion::Explosion;
use oc_root::geo::{WorldVec2, WorldVec3};
use oc_root::physics::Meters;
use oc_root::{Client, TILE_SALT_INTERIOR_DESTRUCTION, WcfgFrom, WcfgInto, WorldConfig};
use oc_utils::let_some;
use oc_utils::polygon::Polygon;
use oc_world::World;
use oc_world::crater::Crater;
use oc_world::interior::Interior;
use oc_world::smoke::SmokeCloud;
use oc_world::tile::AlteredTile;
use rustc_hash::FxHashSet;

use super::update::{ExplosionOrigin, Update};
use crate::individual::death::die;
use crate::routing::Listening;
use oc_network::ToClient;

/// Alters used when an interior no define `destruction_alter_natures`
const DEFAULT_INTERIOR_DESTRUCTION_ALTERS: [Alter; 4] = [
    Alter::BrickDebris,
    Alter::MixedDebris,
    Alter::StoneDebris,
    Alter::WoodDebris,
];

/// Rayon of the smoke cloud made by a smoke grenade
const SMOKE_RAYON: Meters = Meters(6.0);
/// Smoke opacity at the center of the cloud
const SMOKE_CENTER_OPACITY: Percent = Percent(100);
/// Smoke opacity at the edge of the cloud
const SMOKE_EDGE_OPACITY: Percent = Percent(43);

/// Suppress value added to survivors in the explosion suppress rayon
const EXPLOSION_SUPPRESS_INCREASE: u8 = 75;

/// Number of crater sprites available per crater type in the crater sprites
const CRATER_SPRITES_COUNT: usize = 5;

impl<E: Client> super::State<E> {
    // TODO: split in multiple functions
    pub fn spawn_explosion(
        &self,
        explosion: Explosion,
        position: WorldVec2,
        origin: ExplosionOrigin,
    ) -> Vec<(Listening, Vec<ToClient>)> {
        let (updates, region, debug_fx, sound, largest_rayon, smoke_fx) = {
            let world = self.world();
            let indexes = self.indexes();
            let w = &world.w;
            let mod_ = world.mod_();

            let kill_rayon = explosion.kill_rayon.pixels(w);
            let stunned_rayon = explosion.stunned_rayon.pixels(w);
            let shrapnel_rayon = explosion.shrapnel_rayon.pixels(w);
            let suppress_rayon = explosion.suppress_rayon.pixels(w);
            let largest_rayon = kill_rayon
                .max(stunned_rayon)
                .max(shrapnel_rayon)
                .max(suppress_rayon);
            let largest_visual_rayon = explosion.largest_rayon();
            let is_smoke = origin.is_smoke(mod_);
            let mut smoke_fx = None;

            // Tiles concerned by largest rayon
            let tile = WorldTileIndex::from_(TileXy::from_(position, w), w);
            let_some!(tile_ = world.tile(tile), return vec![]);
            let region: WorldRegionIndex = tile.into_(w);
            let tiles_rayon = (largest_rayon / w.geo_pixels_per_tile() as f32).ceil() as u64;
            let tiles = crate::utils::d2::shape_from_tile(tile, tiles_rayon, w);

            tracing::trace!(name="runner-explosion", position=?position, largest_rayon=largest_rayon);

            // Not dead individuals in these tiles
            let mut seen = FxHashSet::default();
            let individuals: Vec<IndividualIndex> = tiles
                .into_iter()
                .flat_map(|tile| indexes.tile_individuals(tile).iter().copied())
                .filter(|i| seen.insert(*i))
                .filter(|i| !matches!(world.individual(*i).status, Status::Dead))
                .collect();

            let distance = |i: IndividualIndex| {
                let p = world.individual(i).position;
                ((p.x - position.x).powi(2) + (p.y - position.y).powi(2)).sqrt()
            };

            let mut killed = FxHashSet::default();
            let mut updates = vec![];

            // Individuals in kill rayon die, unless protected by an interior
            for i in &individuals {
                if distance(*i) <= kill_rayon
                    && !protected_by_interior(
                        &world,
                        mod_,
                        &explosion,
                        world.individual(*i).position.into(),
                        position,
                    )
                {
                    killed.insert(*i);
                    #[cfg(feature = "battle_instrument")]
                    crate::battle_log::kill(&world, *i, crate::battle_log::Cause::Explosion);
                    updates.extend(die(&world, mod_, *i));
                }
            }

            // Survivors in stunned rayon are stunned
            let stunned_value = w.explosion_stunned_value();
            for i in &individuals {
                if !killed.contains(i) && distance(*i) <= stunned_rayon {
                    updates.push(Update::UpdateIndividual(
                        *i,
                        IndividualUpdate::SetStatus(Status::Stunned(stunned_value)),
                    ));
                }
            }

            // Survivors in suppress rayon are suppressed (except in vehicle, which protects them)
            for i in &individuals {
                if !killed.contains(i)
                    && distance(*i) <= suppress_rayon
                    && world.individual(*i).embarked.is_none()
                {
                    let suppress = world.individual(*i).suppress;
                    updates.push(Update::UpdateIndividual(
                        *i,
                        IndividualUpdate::SetSuppress(oc_root::behavior::Suppress(
                            suppress.0.saturating_add(EXPLOSION_SUPPRESS_INCREASE),
                        )),
                    ));
                }
            }

            // Destructed tiles
            let destructed = destructed_tiles(&world, mod_, &explosion, position);
            if !destructed.is_empty() {
                updates.push(Update::AlterTiles(destructed));
            }

            // Destructed decor
            let removed_decor = destructed_decor(&world, &explosion, position);
            if !removed_decor.is_empty() {
                updates.push(Update::RemoveDecor(removed_decor));
            }

            // Craters
            let crater_size = explosion.destruction_rayon.pixels(w);
            let in_interior = world.interiors.iter().any(|i| i.contains(position));
            let crater_type = tile_nature(&world, tile)
                .filter(|_| !in_interior)
                .and_then(|n| mod_.nature(n).crater);
            if let (true, Some(crater_type)) = (crater_size > 0., crater_type) {
                let sprite = fastrand::usize(0..CRATER_SPRITES_COUNT);
                updates.push(Update::AddCrater(Crater::new(
                    position,
                    crater_type,
                    sprite,
                    crater_size,
                )));
            }

            if let (true, ExplosionOrigin::Grenade(grenade)) = (is_smoke, origin) {
                let grenade = mod_.grenade(grenade);
                // Durations are in game time: shorter in real time when speed is higher
                let speed = w.speed();
                let hold = grenade.smoke_duration / speed;
                let fade = grenade.smoke_fade_duration / speed;
                smoke_fx = Some((hold, fade));
                let tiles = smoke_tiles(&world, position);
                if !tiles.is_empty() {
                    updates.push(Update::AddSmokeCloud(SmokeCloud::new(
                        Instant::now(),
                        Duration::from_secs_f32(hold),
                        Duration::from_secs_f32(fade),
                        tiles,
                    )));
                }
            }

            // Shrapnels
            let start = WorldVec3::new(position.x, position.y, tile_.z_pixels(w));
            let candidates: Vec<IndividualIndex> = individuals
                .iter()
                .copied()
                .filter(|i| !killed.contains(i))
                .collect();
            let (shrapnel_killed, _shrapnel_lines) =
                shrapnels(&world, mod_, &explosion, start, &candidates);
            for i in shrapnel_killed {
                #[cfg(feature = "battle_instrument")]
                crate::battle_log::kill(&world, i, crate::battle_log::Cause::Shrapnel);
                updates.extend(die(&world, mod_, i));
            }

            #[cfg(feature = "debug")]
            let debug_fx = fx::Fx::Visual(fx::Visual::ExplosionDebug(fx::ExplosionDebug {
                position,
                kill_rayon: explosion.kill_rayon,
                stunned_rayon: explosion.stunned_rayon,
                shrapnel_rayon: explosion.shrapnel_rayon,
                destruction_rayon: explosion.destruction_rayon,
                shrapnels: _shrapnel_lines,
            }));
            #[cfg(not(feature = "debug"))]
            let debug_fx = None::<fx::Fx>;

            // Smoke grenade makes no explosion (no sound, sprite nor flash)
            let sound = match origin {
                _ if is_smoke => None,
                ExplosionOrigin::Shell(shell) => mod_.shell(shell).pick_sound(),
                ExplosionOrigin::Grenade(grenade) => mod_.grenade(grenade).pick_sound(),
            }
            .map(|sound| {
                fx::Fx::Audio(fx::Audio::PlayWithGeo(
                    sound,
                    start,
                    fx::SoundBus::Explosion,
                ))
            });

            (
                updates,
                region,
                debug_fx,
                sound,
                Meters(largest_visual_rayon),
                smoke_fx,
            )
        };

        let now = Instant::now();
        for update in updates {
            self.schedule(now, update);
        }

        let mut messages = match smoke_fx {
            Some((hold, fade)) => vec![ToClient::Fx(fx::Fx::Visual(fx::Visual::Smoke(
                position, hold, fade,
            )))],
            None => vec![
                ToClient::Fx(fx::Fx::Visual(fx::Visual::Explosion(
                    position,
                    explosion.stunned_rayon,
                ))),
                ToClient::Fx(fx::Fx::Visual(fx::Visual::Flash(position, largest_rayon))),
            ],
        };
        messages.extend(sound.map(ToClient::Fx));
        #[cfg(feature = "debug")]
        messages.push(ToClient::Fx(debug_fx));
        #[cfg(not(feature = "debug"))]
        let _ = debug_fx;

        vec![(Listening::Regions(vec![region]), messages)]
    }
}

fn destructed_tiles(
    world: &World,
    mod_: &Mod,
    explosion: &Explosion,
    position: WorldVec2,
) -> Vec<AlteredTile> {
    tiles_in_destruction_rayon(world, explosion, position)
        .into_iter()
        .filter_map(|i| destructed_tile(world, mod_, explosion, i))
        .collect()
}

/// Initial opacities of tiles in the smoke cloud
fn smoke_tiles(world: &World, position: WorldVec2) -> Vec<(WorldTileIndex, Percent)> {
    let w = &world.w;
    let rayon = SMOKE_RAYON.pixels(w);
    let tiles_rayon = (rayon / w.geo_pixels_per_tile() as f32).ceil() as u64;
    let center = WorldTileIndex::from_(TileXy::from_(position, w), w);

    crate::utils::d2::shape_from_tile(center, tiles_rayon, w)
        .into_iter()
        .filter_map(|i| {
            let tile_center = tile_center(w, i);
            let (dx, dy) = (tile_center.x - position.x, tile_center.y - position.y);
            let distance = (dx * dx + dy * dy).sqrt();
            if distance > rayon {
                return None;
            }

            let ratio = distance / rayon;
            let (center, edge) = (SMOKE_CENTER_OPACITY.ratio(), SMOKE_EDGE_OPACITY.ratio());
            Some((i, Percent::from_ratio(center + (edge - center) * ratio)))
        })
        .collect()
}

/// Tiles with a decor which resists less than the explosion destruction force, in its destruction
/// rayon
fn destructed_decor(
    world: &World,
    explosion: &Explosion,
    position: WorldVec2,
) -> Vec<WorldTileIndex> {
    tiles_in_destruction_rayon(world, explosion, position)
        .into_iter()
        .filter(|i| {
            world
                .decor
                .at(*i)
                .as_ref()
                .is_some_and(|decor| decor.destruction_resistance < explosion.destruction_force)
        })
        .collect()
}

/// Tiles whose center is in the explosion destruction rayon
fn tiles_in_destruction_rayon(
    world: &World,
    explosion: &Explosion,
    position: WorldVec2,
) -> Vec<WorldTileIndex> {
    let w = &world.w;
    let rayon = explosion.destruction_rayon.pixels(w);
    let pixels_per_tile = w.geo_pixels_per_tile() as f32;
    let center = WorldTileIndex::from_(TileXy::from_(position, w), w);
    let tiles_rayon = (rayon / pixels_per_tile).ceil() as u64;

    crate::utils::d2::shape_from_tile(center, tiles_rayon, w)
        .into_iter()
        .filter(|i| {
            let tile_center = tile_center(w, *i);
            let (dx, dy) = (tile_center.x - position.x, tile_center.y - position.y);
            (dx * dx + dy * dy).sqrt() <= rayon
        })
        .collect()
}

/// Alteration of tile if the explosion destroys it
fn destructed_tile(
    world: &World,
    mod_: &Mod,
    explosion: &Explosion,
    i: WorldTileIndex,
) -> Option<AlteredTile> {
    let nature = mod_.nature(tile_nature(world, i)?);
    let center = tile_center(&world.w, i);

    // If in interior, Interior properties apply
    match world
        .interiors
        .iter()
        .find(|interior| interior.contains(center))
    {
        // Tile in an interior is destructed according to the interior, whatever its own
        // nature (a floor has no `destruction_alter`)
        Some(interior) => (interior.destruction_resistance < explosion.destruction_force)
            .then(|| interior_destruction_alter(mod_, interior, i))
            .flatten()
            .map(|alter| AlteredTile::new(i, alter)),
        None => {
            let alter = nature.destruction_alter?;
            (nature.destruction_resistance < explosion.destruction_force)
                .then_some(AlteredTile::new(i, alter))
        }
    }
}

/// Individual inside an interior is protected when:
/// - explosion is inside this interior and the interior penetration protection superior to explosion destruction force
/// - explosion is outside this interior and a tile on the line between individual and explosion center resists to the
///   explosion destruction force
///
/// An interior is considered as penetrated when explosion is on an altered (destroyed) tile.
fn protected_by_interior(
    world: &World,
    mod_: &Mod,
    explosion: &Explosion,
    individual: WorldVec2,
    position: WorldVec2,
) -> bool {
    let Some(interior) = world.interiors.iter().find(|i| i.contains(individual)) else {
        return false;
    };

    // Penetrated if explosion position is on already destructed tile of this interior. A tile
    // outside the interior (like a wall one) can be destructed without penetrating the interior.
    let explosion_tile = WorldTileIndex::from_(TileXy::from_(position, &world.w), &world.w);
    if world.altered_tiles.at(explosion_tile).is_some()
        && interior.contains(tile_center(&world.w, explosion_tile))
    {
        return false;
    }

    // Individual is protected if interior resist to explosion penetration
    if interior.contains(position) {
        return !interior.penetrated(position, explosion.penetration_force);
    }

    // Test if line between explosion and individual stopped by tile with enough destruction_resistance
    line_tiles(&world.w, position, individual)
        .into_iter()
        .any(|i| {
            tile_destruction_resistance(world, mod_, i)
                .is_some_and(|resistance| resistance >= explosion.destruction_force)
        })
}

/// Nature given to tile destructed in interior: one of the interior ones, else the first nature of
/// one of the default alters (picked by tile index)
fn interior_destruction_alter(
    mod_: &Mod,
    interior: &Interior,
    i: WorldTileIndex,
) -> Option<NatureIndex> {
    match &interior.destruction_alter_natures {
        Some(names) => i
            .pick(names, TILE_SALT_INTERIOR_DESTRUCTION)
            .and_then(|name| mod_.nature_index(name)),
        None => {
            let natures: Vec<NatureIndex> = DEFAULT_INTERIOR_DESTRUCTION_ALTERS
                .iter()
                .map(|alter| mod_.alter_nature(*alter))
                .collect();
            i.pick(&natures, TILE_SALT_INTERIOR_DESTRUCTION).copied()
        }
    }
}

/// Center of tile
fn tile_center(w: &WorldConfig, i: WorldTileIndex) -> WorldVec2 {
    let pixels_per_tile = w.geo_pixels_per_tile() as f32;
    let position = WorldVec2::from_(TileXy::from_(i, w), w);
    WorldVec2::new(
        position.x + pixels_per_tile / 2.,
        position.y + pixels_per_tile / 2.,
    )
}

/// Nature of tile, considering its alteration
fn tile_nature(world: &World, i: WorldTileIndex) -> Option<oc_mod::nature::NatureIndex> {
    let tile = world.tile(i)?;
    Some(match world.altered_tiles.at(i) {
        Some(altered) => altered.nature,
        None => tile.nature,
    })
}

/// Destruction resistance of tile: the strongest of its nature one (a wall) and the one of the
/// interior containing it
fn tile_destruction_resistance(world: &World, mod_: &Mod, i: WorldTileIndex) -> Option<f32> {
    let center = tile_center(&world.w, i);
    let nature = mod_.nature(tile_nature(world, i)?).destruction_resistance;

    Some(
        match world
            .interiors
            .iter()
            .find(|interior| interior.contains(center))
        {
            Some(interior) => nature.max(interior.destruction_resistance),
            None => nature,
        },
    )
}

fn line_tiles(w: &WorldConfig, a: WorldVec2, b: WorldVec2) -> Vec<WorldTileIndex> {
    let start = (a.x.round() as isize, a.y.round() as isize, 0);
    let end = (b.x.round() as isize, b.y.round() as isize, 0);

    let mut tiles: Vec<WorldTileIndex> = vec![];
    for (x, y, _) in Bresenham3d::new(start, end) {
        let tile = WorldTileIndex::from_(TileXy::from_(WorldVec2::new(x as f32, y as f32), w), w);
        if tiles.last() != Some(&tile) {
            tiles.push(tile);
        }
    }

    tiles
}

// WARNING: AI generated
// TODO: verify this code
/// Individuals killed by explosion's shrapnels. Individuals inside an interior or a vehicle are
/// not concerned.
fn shrapnels(
    world: &World,
    mod_: &Mod,
    explosion: &Explosion,
    start: WorldVec3,
    candidates: &[IndividualIndex],
) -> (FxHashSet<IndividualIndex>, Vec<(WorldVec2, WorldVec2)>) {
    let w = &world.w;
    let shrapnel_rayon = explosion.shrapnel_rayon.pixels(w);
    let distance = |i: IndividualIndex| {
        let p = world.individual(i).position;
        ((p.x - start.x).powi(2) + (p.y - start.y).powi(2)).sqrt()
    };
    let candidates: Vec<IndividualIndex> = candidates
        .iter()
        .copied()
        .filter(|i| distance(*i) <= shrapnel_rayon)
        // Individuals in vehicle are protected against shrapnels
        .filter(|i| world.individual(*i).embarked.is_none())
        .filter(|i| {
            let p = world.individual(*i).position.into();
            !world.interiors.iter().any(|interior| interior.contains(p))
        })
        .collect();
    let mut killed = FxHashSet::default();
    let mut lines = Vec::with_capacity(explosion.shrapnel_quantity as usize);

    for _ in 0..explosion.shrapnel_quantity {
        let angle = fastrand::f32() * std::f32::consts::TAU;
        let length = shrapnel_rayon * fastrand::f32();
        let end = WorldVec3::new(
            start.x + length * angle.cos(),
            start.y + length * angle.sin(),
            start.z,
        );

        lines.push((
            WorldVec2::new(start.x, start.y),
            WorldVec2::new(end.x, end.y),
        ));

        // Individual is killed only if on the line of shrapnel and no tile stop it before
        let tiles = line_tiles(
            w,
            WorldVec2::new(start.x, start.y),
            WorldVec2::new(end.x, end.y),
        );
        for i in &candidates {
            let p = world.individual(*i).position;
            let tile = WorldTileIndex::from_(TileXy::from_(WorldVec2::new(p.x, p.y), w), w);
            let Some(reached) = tiles.iter().position(|t| *t == tile) else {
                continue;
            };

            if !shrapnel_stopped(world, mod_, start, &tiles[..reached]) {
                killed.insert(*i);
            }
        }
    }

    (killed, lines)
}

// TODO: Shrapnels ignored intinterior or not ?
/// Test if shrapnel stopped by one of `tiles` on the path (like projectiles). Tile of start is ignored.
fn shrapnel_stopped(world: &World, mod_: &Mod, start: WorldVec3, tiles: &[WorldTileIndex]) -> bool {
    let w = &world.w;

    tiles.iter().skip(1).any(|i| {
        let Some(tile) = world.tile(*i) else {
            return false;
        };
        let Some(nature) = tile_nature(world, *i).map(|n| mod_.nature(n)) else {
            return false;
        };

        let ground = tile.z_pixels(w);
        if start.z < ground {
            return true;
        }

        let top = ground + nature.z.0 * w.geo_pixels_per_meters();
        if start.z > top {
            return false;
        }

        !nature.traversability.projectile
            || fastrand::f32() < nature.intercept.projectile.probability()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shrapnels_spare_individuals_in_vehicle() {
        // Given: two individuals near explosion, one of them embarked in a vehicle
        let w = WorldConfig::new(100, 100);
        let position = WorldVec3::new(260., 250., 0.);
        let individual = ::tests::individual::TestIndividual::builder()
            .position(position)
            .build();
        let embarked = ::tests::individual::TestIndividual::builder()
            .position(position)
            .build()
            .make(&w)
            .with_embarked(Some((
                oc_vehicle::VehicleIndex(0),
                oc_mod::vehicle::CrewRole::Driver,
            )));
        let world = ::tests::world::TestWorld::builder()
            .individuals(vec![individual.make(&w), embarked])
            .build()
            .make(&w);
        let explosion = Explosion {
            kill_rayon: Meters(0.),
            stunned_rayon: Meters(0.),
            shrapnel_rayon: Meters(10.),
            suppress_rayon: Meters(0.),
            destruction_rayon: Meters(0.),
            destruction_force: 0.,
            shrapnel_quantity: 5000,
            penetration_force: 0.,
        };
        let start = WorldVec3::new(250., 250., 0.);
        let candidates = [IndividualIndex(0), IndividualIndex(1)];

        // When
        let (killed, _) = shrapnels(&world, world.mod_(), &explosion, start, &candidates);

        // Then
        assert!(killed.contains(&IndividualIndex(0)));
        assert!(!killed.contains(&IndividualIndex(1)));
    }

    #[test]
    fn test_line_tiles() {
        let w = WorldConfig::new(100, 100);
        let ppt = w.geo_pixels_per_tile() as f32;
        let tile_at =
            |x: f32, y: f32| WorldTileIndex::from_(TileXy::from_(WorldVec2::new(x, y), &w), &w);

        // Horizontal line crosses 4 consecutive tiles, without duplicates
        let tiles = line_tiles(
            &w,
            WorldVec2::new(ppt * 0.5, ppt * 0.5),
            WorldVec2::new(ppt * 3.5, ppt * 0.5),
        );
        assert_eq!(
            tiles,
            (0..4)
                .map(|x| tile_at(ppt * (x as f32 + 0.5), ppt * 0.5))
                .collect::<Vec<_>>()
        );

        // Same position: only one tile
        let p = WorldVec2::new(ppt * 2.5, ppt * 2.5);
        assert_eq!(line_tiles(&w, p, p), vec![tile_at(p.x, p.y)]);

        // Line starts and ends on given tiles
        let (a, b) = (WorldVec2::new(1., 1.), WorldVec2::new(ppt * 5., ppt * 3.));
        let tiles = line_tiles(&w, a, b);
        assert_eq!(tiles.first(), Some(&tile_at(a.x, a.y)));
        assert_eq!(tiles.last(), Some(&tile_at(b.x, b.y)));
    }
}
