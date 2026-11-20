use bevy::prelude::*;
use oc_geo::{
    region::{RegionXy, WorldRegionIndex},
    tile::{TileXy, WorldHeightIndex, WorldTileIndex},
};
use oc_individual::{
    Individual, IndividualIndex,
    squad::{Squad, SquadIndex},
};
use oc_mod::Mod;
use oc_physics::Physic;
#[cfg(feature = "debug")]
use oc_root::physics::Meters;
use oc_root::{
    WcfgFrom, WcfgInto, WorldConfig,
    geo::{WorldVec2, WorldVec3},
};
use oc_utils::let_some;
use oc_utils::percent::Percent;
use oc_vehicle::{Vehicle, VehicleIndex};
use oc_world::{
    crater::Crater,
    decor::Decor,
    support::SupportType,
    tile::{AlteredTile, Tile},
};
use rustc_hash::FxHashMap;

use crate::{
    ingame::{
        BattlePhase, WorldResumeEvent, behavior::SpawnSquadOrders, camera::GoToPoint,
        physics::ObjectId, spawn::ShowSpawnZones,
    },
    states::GameConfig,
    world::{path::navmesh, visibilities::Visibilities},
};

pub mod individual;
pub mod path;
pub mod tile;
pub mod vehicle;
pub mod visibilities;

#[derive(Debug, Event)]
pub struct InsertTiles(pub WorldRegionIndex, pub Vec<(WorldTileIndex, Tile)>);

#[derive(Debug, Event)]
pub struct UpdateSmoke(pub WorldRegionIndex, pub Vec<(WorldTileIndex, Percent)>);

#[derive(Debug, Event)]
pub struct InsertAlteredTiles(pub WorldRegionIndex, pub Vec<AlteredTile>);

#[derive(Debug, Event)]
pub struct InsertDecor(pub WorldRegionIndex, pub Vec<Decor>);

#[derive(Debug, Event)]
pub struct RemoveDecor(pub WorldRegionIndex, pub Vec<WorldTileIndex>);

#[derive(Debug, Event)]
pub struct InsertCraters(pub WorldRegionIndex, pub Vec<Crater>);

#[derive(Debug, Event)]
pub struct UpdateVisibilities(
    pub  Vec<(
        IndividualIndex,
        IndividualIndex,
        oc_world::visibility::Visibility,
    )>,
);

#[derive(Debug, Event)]
pub struct VisibilitiesUpdated;

#[allow(unused)]
#[derive(Debug, Event)]
pub struct InsertedTiles(pub WorldRegionIndex);

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<World>()
            .add_observer(on_world_resume)
            .add_observer(tile::on_insert_tiles)
            .add_observer(tile::on_insert_altered_tiles)
            .add_observer(tile::on_insert_decor)
            .add_observer(tile::on_remove_decor)
            .add_observer(tile::on_update_smoke)
            .add_observer(tile::on_forgotten_region)
            .add_observer(individual::on_insert_individual)
            .add_observer(individual::on_update_individual_physics)
            .add_observer(individual::on_forgotten_region)
            .add_observer(vehicle::on_insert_vehicle)
            .add_observer(vehicle::on_forgot_vehicle)
            .add_observer(vehicle::on_forgotten_region)
            .add_observer(visibilities::on_update_visibilities)
            .add_observer(visibilities::on_visibilities_updated);
    }
}

#[derive(Debug, Deref, DerefMut)]
pub struct Index<K: std::fmt::Debug, V: std::fmt::Debug>(
    FxHashMap<WorldRegionIndex, FxHashMap<K, V>>,
);

impl<K: std::fmt::Debug, V: std::fmt::Debug> Default for Index<K, V> {
    fn default() -> Self {
        Self(FxHashMap::default())
    }
}

// FIXME: improve perfs by using Partial and list index (like in previous projects)
#[derive(Resource, Default)]
pub struct World {
    pub individuals: Index<WorldTileIndex, Vec<(IndividualIndex, Individual)>>,
    pub individuals_refs: FxHashMap<IndividualIndex, (WorldRegionIndex, WorldTileIndex)>, // TODO: remove pub and ensure x_ref
    pub vehicles: Index<VehicleIndex, Vehicle>,
    pub vehicles_refs: FxHashMap<VehicleIndex, WorldRegionIndex>,
    pub tiles: Index<WorldTileIndex, Tile>,
    pub altered_tiles: Index<WorldTileIndex, AlteredTile>,
    pub decor: Index<WorldTileIndex, Decor>,
    /// Smoke opacity (255 is 1.0) of tiles with smoke
    pub smoke: Index<WorldTileIndex, Percent>,
    pub heights: Index<WorldHeightIndex, u8>,
    pub terrain: Option<oc_world::terrain::Terrain>,
    pub squads: Index<SquadIndex, Squad>,
    pub squads_refs: FxHashMap<SquadIndex, WorldRegionIndex>, // TODO: remove pub and ensure x_ref
    pub individual_squad: FxHashMap<IndividualIndex, SquadIndex>,
    pub supports: Vec<(SupportType, u8)>,
    // TODO: is that too much blocking to recompute it when world change ?
    // do it asynchronously ? Or compute paths server side ?
    navmesh: polyanya::Mesh,
    // FIXME: something with index ? See usage
    pub visibilities: Visibilities,
}

impl World {
    pub fn insert_tiles(&mut self, region: WorldRegionIndex, tiles: Vec<(WorldTileIndex, Tile)>) {
        let heights: Vec<(WorldHeightIndex, u8)> =
            tiles.iter().map(|(i, t)| ((*i).into(), t.z)).collect();
        self.tiles
            .entry(region)
            .and_modify(|tiles_| {
                // TODO: Damn, .clone() is pain here !
                for (i, tile) in tiles.clone().into_iter() {
                    tiles_.insert(i, tile);
                }
            })
            .or_insert(tiles.into_iter().collect());
        self.heights
            .entry(region)
            .and_modify(|heights_| {
                // TODO: Damn, .clone() is pain here !
                for (i, tile) in heights.clone().into_iter() {
                    heights_.insert(i, tile);
                }
            })
            .or_insert(heights.into_iter().collect());
    }

    pub fn insert_altered_tiles(
        &mut self,
        region: WorldRegionIndex,
        tiles: Vec<AlteredTile>,
        mod_: &Mod,
    ) {
        self.altered_tiles
            .entry(region)
            .or_default()
            .extend(tiles.into_iter().map(|tile| (tile.i, tile)));
        self.apply_altered_tiles(region, mod_);
    }

    pub fn apply_altered_tiles(&mut self, region: WorldRegionIndex, mod_: &Mod) {
        let_some!(tiles = self.tiles.get_mut(&region), return);
        let_some!(altered = self.altered_tiles.get(&region), return);

        for (i, altered) in altered {
            if let Some(tile) = tiles.get_mut(i) {
                tile.alter(altered, mod_);
            }
        }
    }

    pub fn insert_decor(&mut self, region: WorldRegionIndex, decor: Vec<Decor>) {
        self.decor
            .entry(region)
            .or_default()
            .extend(decor.into_iter().map(|decor| (decor.i, decor)));
    }

    pub fn remove_decor(&mut self, region: WorldRegionIndex, tiles: &[WorldTileIndex]) {
        let_some!(decor = self.decor.get_mut(&region), return);
        for i in tiles {
            decor.remove(i);
        }
    }

    pub fn update_smoke(&mut self, region: WorldRegionIndex, smoke: &[(WorldTileIndex, Percent)]) {
        let region_smoke = self.smoke.entry(region).or_default();
        for (i, opacity) in smoke {
            match opacity.is_zero() {
                true => region_smoke.remove(i),
                false => region_smoke.insert(*i, *opacity),
            };
        }
    }

    pub fn smoke_at(&self, xy: TileXy, w: &WorldConfig) -> Percent {
        let i = WorldTileIndex::from_(xy, w);
        let region: WorldRegionIndex = i.into_(w);
        self.smoke
            .get(&region)
            .and_then(|smoke| smoke.get(&i))
            .copied()
            .unwrap_or_default()
    }

    pub fn update_navmesh(&mut self, w: &WorldConfig, mod_: &Mod) {
        let tiles: Vec<(&WorldTileIndex, &Tile)> =
            self.tiles.values().flat_map(|inner| inner.iter()).collect();
        self.navmesh = navmesh(w, mod_, tiles);
    }

    pub fn remove_tiles(&mut self, region: WorldRegionIndex) {
        self.tiles.remove(&region);
        self.altered_tiles.remove(&region);
        self.decor.remove(&region);
        self.smoke.remove(&region);
    }

    pub fn insert_individual(
        &mut self,
        w: &WorldConfig,
        i: IndividualIndex,
        individual: Individual,
    ) {
        let position = individual.position(w);
        let tile_xy: TileXy = position.into_(w);
        let tile: WorldTileIndex = tile_xy.into_(w);
        let region: WorldRegionIndex = tile.into_(w);

        tracing::trace!(name = "world-individual-insert", i=?i, region=?region, tile=?tile);

        let value = (i, individual);
        self.individuals
            .entry(region)
            .and_modify(|tiles| {
                tiles
                    .entry(tile)
                    .and_modify(|individuals| individuals.push(value.clone()))
                    .or_insert(vec![value.clone()]);
            })
            .or_insert(FxHashMap::from_iter(vec![(tile, vec![value])]));
        self.individuals_refs.insert(i, (region, tile));
    }

    pub fn remove_individual(&mut self, w: &WorldConfig, i: IndividualIndex, position: WorldVec3) {
        let position = TileXy::from_(position, w);
        let tile: WorldTileIndex = position.into_(w);
        let region: WorldRegionIndex = tile.into_(w);

        if let Some(tiles) = self.individuals.get_mut(&region)
            && let Some(individuals) = tiles.get_mut(&tile)
        {
            individuals.retain(|(i_, _)| *i_ != i);
        }
        self.individuals_refs.remove(&i);
    }

    pub fn insert_vehicle(&mut self, w: &WorldConfig, i: VehicleIndex, vehicle: Vehicle) {
        let tile_xy: TileXy = vehicle.position.into_(w);
        let tile: WorldTileIndex = tile_xy.into_(w);
        let region: WorldRegionIndex = tile.into_(w);

        // Vehicle may have changed of region
        self.remove_vehicle(i);
        self.vehicles.entry(region).or_default().insert(i, vehicle);
        self.vehicles_refs.insert(i, region);
    }

    pub fn remove_vehicle(&mut self, i: VehicleIndex) {
        let_some!(region = self.vehicles_refs.remove(&i), return);
        if let Some(vehicles) = self.vehicles.get_mut(&region) {
            vehicles.remove(&i);
        }
    }

    pub fn remove_vehicles(&mut self, region: WorldRegionIndex) {
        if let Some(vehicles) = self.vehicles.remove(&region) {
            for i in vehicles.keys() {
                self.vehicles_refs.remove(i);
            }
        }
    }

    pub fn vehicle(&self, i: VehicleIndex) -> Option<&Vehicle> {
        let region = self.vehicles_refs.get(&i)?;
        self.vehicles
            .get(region)
            .and_then(|vehicles| vehicles.get(&i))
    }

    // TODO: if consume too much CPU: create an index
    /// Known individuals embarked in vehicle `i`
    pub fn vehicle_crew(&self, i: VehicleIndex) -> Vec<IndividualIndex> {
        self.individuals
            .values()
            .flat_map(|tiles| tiles.values())
            .flatten()
            .filter(|(_, individual)| individual.embarked.is_some_and(|(i_, _)| i_ == i))
            .map(|(i, _)| *i)
            .collect()
    }

    pub fn remove_individuals(&mut self, region: WorldRegionIndex) {
        self.individuals.remove(&region);
    }

    pub fn individual(&self, i: IndividualIndex) -> Option<&Individual> {
        let (region, tile) = self.individuals_refs.get(&i)?;

        self.individuals
            .get(region)
            .and_then(|region| region.get(tile))
            .and_then(|individuals| {
                individuals
                    .iter()
                    .find_map(|(i_, individual)| (*i_ == i).then_some(individual))
            })
    }

    pub fn get_individual_mut(&mut self, i: IndividualIndex) -> Option<&mut Individual> {
        let (region, tile) = self.individuals_refs.get(&i)?;

        self.individuals
            .get_mut(region)
            .and_then(|region| region.get_mut(tile))
            .and_then(|individuals| {
                individuals
                    .iter_mut()
                    .find_map(|(i_, individual)| (*i_ == i).then_some(individual))
            })
    }

    // TODO: This is not optimal way ... Goal is remove individual from old index (region index) and insert in new one
    pub fn refresh_individual_position(
        &mut self,
        w: &WorldConfig,
        i: IndividualIndex,
        position: WorldVec3,
    ) {
        let_some!(individual = self.individual(i).cloned(), return);
        self.remove_individual(w, i, position);
        self.insert_individual(w, i, individual.clone());
    }

    /// An individual can be engaged if it is of the given `side`, visible and matching status
    pub fn engageable(&self, i: IndividualIndex, side: oc_root::side::Side) -> bool {
        self.individual(i)
            .is_some_and(|individual| individual.side == side && individual.status.is_target())
            && self.visible(i)
    }

    /// Closest engageable individual of `side` within `radius` of `point`
    pub fn individual_nearest(
        &self,
        point: WorldVec2,
        radius: f32,
        side: oc_root::side::Side,
    ) -> Option<IndividualIndex> {
        let distance = |individual: &Individual| {
            let (dx, dy) = (
                individual.position.x - point.x,
                individual.position.y - point.y,
            );
            dx * dx + dy * dy
        };

        self.individuals
            .values()
            .flat_map(|tiles| tiles.values().flatten())
            .map(|(i, individual)| (*i, distance(individual)))
            // In the allowed radius to be considered as near
            .filter(|&(_, d)| d <= radius * radius)
            .filter(|&(i, _)| self.engageable(i, side))
            // Take the nearest
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(i, _)| i)
    }

    pub fn at(&self, w: &WorldConfig, tile: TileXy) -> Vec<(ObjectId, &dyn Physic)> {
        let region: WorldRegionIndex = tile.into_(w);
        let tile: WorldTileIndex = tile.into_(w);

        let mut objects = vec![];

        objects = self
            .individuals
            .get(&region)
            .and_then(|region| region.get(&tile))
            .map(|individuals| {
                individuals
                    .iter()
                    .map(|(i, individual)| {
                        let individual: &dyn Physic = individual;
                        (ObjectId::Individual(*i), individual)
                    })
                    .collect::<Vec<(ObjectId, &dyn Physic)>>()
            })
            .unwrap_or_default();

        if let Some(tile_) = self.tiles.get(&region).and_then(|tiles| tiles.get(&tile)) {
            let tile_: &dyn Physic = tile_;
            objects.push((ObjectId::Tile(tile), tile_));
        }

        objects
    }

    pub fn tile(&self, w: &WorldConfig, xy: TileXy) -> Option<&Tile> {
        let i: WorldTileIndex = xy.into_(w);
        let region: WorldRegionIndex = i.into_(w);
        self.tiles.get(&region).and_then(|tiles| tiles.get(&i))
    }

    pub fn tile_at(&self, w: &WorldConfig, point: WorldVec2) -> Option<&Tile> {
        let xy = TileXy::from_([point.x, point.y], w);
        self.tile(w, xy)
    }

    #[cfg(feature = "debug")]
    pub fn tiles(&self) -> &Index<WorldTileIndex, Tile> {
        &self.tiles
    }

    #[cfg(feature = "debug")]
    pub fn heights(&self) -> &Index<WorldHeightIndex, u8> {
        &self.heights
    }

    #[cfg(feature = "debug")]
    pub fn d2_to_d3(&self, w: &WorldConfig, p: WorldVec2, plus_z: Meters) -> Option<WorldVec3> {
        let tile = TileXy::from_(p, w);
        let tile = self.tile(w, tile)?;
        let z =
            (tile.z as f32 * w.geo_meters_per_z().0 * w.geo_pixels_per_meters()) + plus_z.pixels(w);
        let p = [p.x, p.y, z];
        Some(p.into())
    }

    pub fn squads(&self) -> impl Iterator<Item = SquadIndex> + '_ {
        self.squads
            .values()
            .flat_map(|squads| squads.keys().copied())
    }

    pub fn squad(&self, i: SquadIndex) -> Option<&Squad> {
        let region = self.squads_refs.get(&i)?;
        let squads = self.squads.get(region)?;
        squads.get(&i)
    }

    pub fn individual_squad(&self, i: IndividualIndex) -> Option<(SquadIndex, &Squad)> {
        let i = self.individual_squad.get(&i)?;
        let region = self.squads_refs.get(i)?;
        let squads = self.squads.get(region)?;
        let squad = squads.get(i)?;
        Some((*i, squad))
    }

    pub fn path(&self, from: WorldVec2, to: WorldVec2) -> Option<polyanya::Path> {
        self.navmesh.path([from.x, from.y], [to.x, to.y])
    }

    pub fn side_squads(
        &self,
        side: oc_root::side::Side,
    ) -> impl Iterator<Item = (SquadIndex, &Squad)> + '_ {
        self.squads
            .values()
            .flat_map(|squads| squads.iter())
            .filter(move |(_, squad)| squad.side == side)
            .map(|(i, squad)| (*i, squad))
    }

    pub fn operational(&self, i: IndividualIndex) -> bool {
        self.individual(i)
            .is_some_and(|individual| individual.status == oc_individual::Status::Operational)
    }

    pub fn visible(&self, i: IndividualIndex) -> bool {
        self.visibilities.visible.contains(&i)
    }
}

fn on_world_resume(
    event: On<WorldResumeEvent>,
    g_res: Res<GameConfig>,
    mut world: ResMut<World>,
    mut commands: Commands,
    mut phase: ResMut<NextState<BattlePhase>>,
    network: Res<crate::network::state::State>,
) {
    let_some!(g = &g_res.0, return);
    let_some!(identity = &network.identity, return);

    world.supports = event.0.support.clone();

    for (i, squad) in &event.0.squads {
        let position = squad.position;
        let tile_xy = TileXy::from_(position, &g.w);
        let region = RegionXy::from_(tile_xy, &g.w);
        let region = WorldRegionIndex::from_(region, &g.w);

        world
            .squads
            .entry(region)
            .or_default()
            .insert(*i, squad.clone());
        world.squads_refs.insert(*i, region);
        let own = squad.side == identity.side;

        // Opposite squads are only known to display them on minimap. In debug, also index their
        // members to permit selecting them.
        if squad.side == identity.side || cfg!(feature = "debug") {
            for member in &squad.members {
                world.individual_squad.insert(*member, *i);
            }
        }
        if !own {
            continue;
        }

        tracing::trace!(name="world-on-world-resume-trigger-spawn-squad-orders", i=?i, orders=?squad.orders);
        commands.trigger(SpawnSquadOrders(*i, squad.orders.clone()));

        // Move screen to first found squad
        let_some!((_, squad) = world.side_squads(identity.side).next(), return);
        commands.trigger(GoToPoint(squad.position));
    }

    commands.trigger(ShowSpawnZones(
        event.0.spawns.a.clone(),
        event.0.spawns.b.clone(),
    ));
    let phase_ = match event.0.phase {
        oc_root::battle::BattlePhase::Deployment => BattlePhase::Deployment,
        oc_root::battle::BattlePhase::Fight => BattlePhase::Fight,
        oc_root::battle::BattlePhase::End(_) => BattlePhase::End,
    };
    *phase = NextState::Pending(phase_);
    crate::fx::play_phase_start(&mut commands, &g_res, phase_);
}
