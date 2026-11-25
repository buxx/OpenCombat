use std::collections::HashMap;

use getset::WithSetters;
use glam::Vec2;
use oc_geo::tile::TileXy;
use oc_mod::vehicle::VehicleIndex;
use oc_root::uuids::SquadUuid;
use oc_root::{WcfgFrom, WorldConfig, material::MaterialKind};
use oc_root::{geo::WorldVec2, side::BySide};
use oc_utils::index::AtIndex;
use oc_utils::{
    d2::{Direction, Xy},
    polygon::Polygon,
};
use oc_world::{spawn::SpawnZone, tile::Tile, utils::oriented_rectangle_covered_tiles};

#[derive(Debug, WithSetters)]
pub struct Placer {
    places: Vec<WorldVec2>,
    assigned: HashMap<SquadUuid, WorldVec2>,
    #[getset(set_with = "pub")]
    orders: BySide<OrderPolicy>,
}

impl Placer {
    pub fn new(places: Vec<WorldVec2>) -> Self {
        Self {
            places,
            assigned: HashMap::default(),
            orders: BySide::default(),
        }
    }

    pub fn with_assigned(mut self, places: Vec<(SquadUuid, WorldVec2)>) -> Self {
        self.assigned = places.into_iter().collect();
        self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn place(
        &self,
        w: &WorldConfig,
        mod_: &oc_mod::Mod,
        tiles: &[Tile],
        spawns: &[SpawnZone],
        uuid: SquadUuid,
        occupied: &[WorldVec2],
        policy: &PlacementPolicy,
    ) -> WorldVec2 {
        let fits = |p: WorldVec2| policy.fits(w, mod_, tiles, spawns, p);

        // If placer already know a place for the squad, us it
        // (only if it is in an owned spawn zone)
        if let Some(place) = self.assigned.get(&uuid)
            && fits(*place)
        {
            return *place;
        }

        // Elsewhere, pick randomly a free place in available spawn zone
        let candidates: Vec<&WorldVec2> = self
            .places
            .iter()
            .filter(|p| policy.free(w, mod_, **p, occupied) && fits(**p))
            .collect();
        if let Some(place) = fastrand::choice(candidates) {
            return *place;
        }

        // No free place: vehicles search a free place where their chassis fit
        if matches!(policy, PlacementPolicy::Vehicle { .. }) {
            let candidates: Vec<WorldVec2> = spawns
                .iter()
                .flat_map(|s| tiles_centers(w, s))
                .filter(|p| policy.free(w, mod_, *p, occupied) && fits(*p))
                .collect();
            if let Some(place) = fastrand::choice(candidates) {
                return place;
            }
        }

        // No free place: use the start of an owned spawn zone
        spawns
            .first()
            .map(|s| s.start)
            .expect("Side must own at least one spawn zone")
    }

    pub fn orders(
        &self,
        _w: &oc_root::WorldConfig,
        _mod_: &oc_mod::Mod,
        snapshot: &oc_world::snapshot::Snapshot,
        squad: &oc_individual::squad::Squad,
    ) -> Vec<oc_individual::order::Order> {
        let policy = &self.orders[squad.side];

        if matches!(policy, OrderPolicy::None) {
            return vec![];
        }

        let opposite_side = squad.side.opposite();
        let nearest_enemy = snapshot
            .individuals
            .iter()
            .filter(|individual| individual.side == opposite_side)
            .min_by(|a, b| {
                let da = WorldVec2::from(a.position) - squad.position;
                let db = WorldVec2::from(b.position) - squad.position;
                (da.x * da.x + da.y * da.y).total_cmp(&(db.x * db.x + db.y * db.y))
            });

        let direction = match nearest_enemy {
            Some(individual) => {
                Direction::from_points2d(squad.position.into(), individual.position.into())
            }
            None => oc_utils::d2::Direction::default(),
        };

        vec![match policy {
            OrderPolicy::None => unreachable!(),
            OrderPolicy::Hide => oc_individual::order::Order::Hide(direction),
            OrderPolicy::Defend => oc_individual::order::Order::Defend(direction),
        }]
    }
}

/// How a squad is placed on the map
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlacementPolicy {
    Squad,
    Vehicle {
        type_: VehicleIndex,
        direction: Direction,
    },
}

impl From<&oc_vehicle::Vehicle> for PlacementPolicy {
    fn from(vehicle: &oc_vehicle::Vehicle) -> Self {
        Self::Vehicle {
            type_: vehicle.type_,
            direction: vehicle.chassis_direction,
        }
    }
}

impl PlacementPolicy {
    /// True if squad can be placed at `position` regarding owned `spawns` and `tiles`
    pub fn fits(
        &self,
        w: &WorldConfig,
        mod_: &oc_mod::Mod,
        tiles: &[Tile],
        spawns: &[SpawnZone],
        position: WorldVec2,
    ) -> bool {
        let inside = |p: WorldVec2| spawns.iter().any(|s| s.contains(p));
        match self {
            PlacementPolicy::Squad => inside(position),
            PlacementPolicy::Vehicle { type_, direction } => {
                let (width, length) = mod_.vehicle(*type_).size;
                let (width, length) = (width.pixels(w), length.pixels(w));
                chassis_corners(position, width, length, *direction)
                    .into_iter()
                    .all(inside)
                    && oriented_rectangle_covered_tiles(
                        [position.x, position.y],
                        width,
                        length,
                        *direction,
                        w,
                    )
                    .into_iter()
                    .all(|i| !tiles.at(i).allow.deny(MaterialKind::Vehicle))
            }
        }
    }

    /// True if `position` is not too close of `occupied` places
    pub fn free(
        &self,
        w: &WorldConfig,
        mod_: &oc_mod::Mod,
        position: WorldVec2,
        occupied: &[WorldVec2],
    ) -> bool {
        match self {
            PlacementPolicy::Squad => !occupied.contains(&position),
            PlacementPolicy::Vehicle { type_, .. } => {
                let size = mod_.vehicle(*type_).size;
                // Considering the chassis can be rotated, keep at least its length from others
                let length = size.0.pixels(w).max(size.1.pixels(w));
                let position = Vec2::from(position);
                occupied
                    .iter()
                    .all(|o| Vec2::from(*o).distance(position) >= length)
            }
        }
    }
}

/// Corners of a `width` x `length` rectangle centered on `center` and oriented along `direction`
fn chassis_corners(
    center: WorldVec2,
    width: f32,
    length: f32,
    direction: Direction,
) -> [WorldVec2; 4] {
    let center = Vec2::from(center);
    let forward = Vec2::new(direction.x, direction.y)
        .try_normalize()
        .unwrap_or(Vec2::new(Direction::NORTH.x, Direction::NORTH.y));
    let lateral = forward.perp();
    let (forward, lateral) = (forward * length / 2., lateral * width / 2.);
    [
        center + forward + lateral,
        center + forward - lateral,
        center - forward + lateral,
        center - forward - lateral,
    ]
    .map(|p| WorldVec2::new(p.x, p.y))
}

/// Center of tiles inside the bounding box of `zone`
fn tiles_centers(w: &WorldConfig, zone: &SpawnZone) -> Vec<WorldVec2> {
    let mut points = zone.points.iter().map(|p| Vec2::from(*p).max(Vec2::ZERO));
    let Some(first) = points.next() else {
        return vec![];
    };
    let (min, max) = points.fold((first, first), |(min, max), p| (min.min(p), max.max(p)));
    let TileXy(Xy(min_x, min_y)) = TileXy::from_(WorldVec2::new(min.x, min.y), w);
    let TileXy(Xy(max_x, max_y)) = TileXy::from_(WorldVec2::new(max.x, max.y), w);
    let max_x = max_x.min(w.world_width().saturating_sub(1));
    let max_y = max_y.min(w.world_height().saturating_sub(1));
    let half = w.geo_pixels_per_tile() as f32 / 2.;

    (min_y..=max_y)
        .flat_map(|y| (min_x..=max_x).map(move |x| TileXy(Xy(x, y))))
        .map(|xy| {
            let [x, y] = xy.point(w);
            WorldVec2::new(x + half, y + half)
        })
        .collect()
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde_derive::Deserialize)]
pub enum OrderPolicy {
    None,
    #[default]
    Hide,
    Defend,
}

impl OrderPolicy {
    /// Name of the policy as command line argument value (like `--order-a hide`)
    pub fn arg_name(&self) -> &'static str {
        match self {
            OrderPolicy::None => "none",
            OrderPolicy::Hide => "hide",
            OrderPolicy::Defend => "defend",
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use oc_geo::tile::WorldTileIndex;
    use oc_mod::{Mod, nature::NatureIndex};
    use oc_world::spawn::SpawnZoneName;

    use super::*;

    /// 30x30 tiles world (1 tile per meter), all tiles allowing vehicles except tile (15, 15)
    fn world() -> (WorldConfig, Vec<Tile>, Mod) {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../mods/tests1");
        let mod_ = Mod::load(&path, None).unwrap();
        let w = WorldConfig::new(30, 30);
        let denied = WorldTileIndex::from_(TileXy(Xy(15, 15)), &w);
        let tiles = (0..w.tiles_count())
            .map(|i| {
                let mut traversability = mod_.nature(NatureIndex(0)).traversability.clone();
                traversability.vehicle = WorldTileIndex(i) != denied;
                Tile::new(WorldTileIndex(i), NatureIndex(0), 0, traversability)
            })
            .collect();
        (w, tiles, mod_)
    }

    fn spawns(w: &WorldConfig) -> Vec<SpawnZone> {
        let size = (w.world_width() * w.geo_pixels_per_tile()) as f32;
        vec![SpawnZone {
            name: SpawnZoneName::new("all"),
            start: WorldVec2::new(0., 0.),
            points: vec![
                WorldVec2::new(0., 0.),
                WorldVec2::new(size, 0.),
                WorldVec2::new(size, size),
                WorldVec2::new(0., size),
            ],
        }]
    }

    fn tile_center(w: &WorldConfig, x: u64, y: u64) -> WorldVec2 {
        let [px, py] = TileXy(Xy(x, y)).point(w);
        let half = w.geo_pixels_per_tile() as f32 / 2.;
        WorldVec2::new(px + half, py + half)
    }

    fn tank(mod_: &Mod) -> PlacementPolicy {
        let type_ = mod_.vehicle_from_name("Tank1").unwrap().index();
        let vehicle = oc_vehicle::Vehicle::new(
            oc_root::side::Side::A,
            type_,
            oc_mod::illustration::IllustrationIndex(0),
            WorldVec2::new(0., 0.).extend(0.),
            vec![],
            Direction::NORTH,
            None,
        );
        PlacementPolicy::from(&vehicle)
    }

    #[test]
    fn vehicle_fits_on_tiles_allowing_vehicles_only() {
        // Given
        let (w, tiles, mod_) = world();
        let spawns = spawns(&w);
        let policy = tank(&mod_);

        // Then: chassis (2.7m x 6.5m) covers the denied tile
        assert!(!policy.fits(&w, &mod_, &tiles, &spawns, tile_center(&w, 15, 15)));
        assert!(!policy.fits(&w, &mod_, &tiles, &spawns, tile_center(&w, 15, 18)));
        // Chassis beside the denied tile
        assert!(policy.fits(&w, &mod_, &tiles, &spawns, tile_center(&w, 15, 20)));
        assert!(policy.fits(&w, &mod_, &tiles, &spawns, tile_center(&w, 5, 15)));
        // Chassis out of spawn zone
        assert!(!policy.fits(&w, &mod_, &tiles, &spawns, tile_center(&w, 5, 1)));
        // Squad ignores chassis
        let squad = PlacementPolicy::Squad;
        assert!(squad.fits(&w, &mod_, &tiles, &spawns, tile_center(&w, 15, 15)));
        assert!(squad.fits(&w, &mod_, &tiles, &spawns, tile_center(&w, 5, 1)));
    }

    #[test]
    fn placer_place_vehicle_where_chassis_fits() {
        // Given
        let (w, tiles, mod_) = world();
        let spawns = spawns(&w);
        let denied = tile_center(&w, 15, 15);
        let allowed = tile_center(&w, 5, 15);
        let placer = Placer::new(vec![denied, allowed]);

        // When
        let squad = SquadUuid::new_v4();
        let position = placer.place(&w, &mod_, &tiles, &spawns, squad, &[], &tank(&mod_));

        // Then
        assert_eq!(position, allowed);
    }

    #[test]
    fn placer_search_vehicle_place_when_no_place_fits() {
        // Given
        let (w, tiles, mod_) = world();
        let spawns = spawns(&w);
        let policy = tank(&mod_);
        let placer = Placer::new(vec![tile_center(&w, 15, 15)]);

        // When
        let squad = SquadUuid::new_v4();
        let position = placer.place(&w, &mod_, &tiles, &spawns, squad, &[], &policy);

        // Then
        assert!(policy.fits(&w, &mod_, &tiles, &spawns, position));
    }
}
