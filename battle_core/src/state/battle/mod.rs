use std::collections::HashMap;

use oc_core::morale::Morale;

use crate::{
    deployment::Deployment,
    entity::{soldier::Soldier, vehicle::Vehicle},
    game::{control::MapControl, flag::FlagsOwnership, Side},
    graphics::vehicle::VehicleGraphicInfos,
    map::Map,
    order::Order,
    physics::{
        event::{bullet::BulletFire, explosion::Explosion},
        path::{Direction, PathMode},
        visibility::Visibilities,
    },
    sync::BattleStateCopy,
    types::{
        SoldierBoard, SoldierIndex, SoldiersOnBoard, SquadComposition, SquadUuid, VehicleBoard,
        VehicleIndex,
    },
    utils::{vehicle_board_from_soldiers_on_board, WorldShape},
};

use self::{
    message::{BattleStateMessage, SideEffect},
    phase::Phase,
};

pub mod builder;
pub mod message;
pub mod order;
pub mod phase;
pub mod soldier;
pub mod squad;
pub mod vehicle;
pub mod visibility;

pub struct BattleState {
    frame_i: u64,
    map: Map,
    phase: Phase,
    soldiers: Vec<Soldier>,
    vehicles: Vec<Vehicle>,
    soldier_on_board: SoldiersOnBoard,
    vehicle_board: VehicleBoard,
    squads: HashMap<SquadUuid, SquadComposition>,
    bullet_fires: Vec<BulletFire>,
    explosions: Vec<Explosion>,
    visibilities: Visibilities,
    a_connected: bool,
    b_connected: bool,
    a_ready: bool,
    b_ready: bool,
    a_morale: Morale,
    b_morale: Morale,
    flags: FlagsOwnership,
}

impl BattleState {
    pub fn new(
        frame_i: u64,
        map: Map,
        soldiers: Vec<Soldier>,
        vehicles: Vec<Vehicle>,
        soldier_on_board: SoldiersOnBoard,
        phase: Phase,
        flags: FlagsOwnership,
    ) -> Self {
        let vehicle_board = vehicle_board_from_soldiers_on_board(&soldier_on_board);
        Self {
            frame_i,
            map,
            phase,
            soldiers,
            vehicles,
            soldier_on_board,
            vehicle_board,
            squads: HashMap::new(),
            bullet_fires: vec![],
            explosions: vec![],
            visibilities: Visibilities::default(),
            a_connected: false,
            b_connected: false,
            a_ready: false,
            b_ready: false,
            a_morale: Morale(1.0), // FIXME BS NOW : from context ?
            b_morale: Morale(1.0), // FIXME BS NOW : from context ?
            flags,
        }
    }

    pub fn empty(map: &Map) -> Self {
        Self {
            frame_i: 0,
            map: map.clone(),
            phase: Phase::Placement,
            soldiers: vec![],
            vehicles: vec![],
            soldier_on_board: HashMap::new(),
            vehicle_board: HashMap::new(),
            squads: HashMap::new(),
            bullet_fires: vec![],
            explosions: vec![],
            visibilities: Visibilities::default(),
            a_connected: false, // TODO : should be in (server) Runner ?
            b_connected: false, // TODO : should be in (server) Runner ?
            a_ready: false,
            b_ready: false,
            a_morale: Morale(1.0),
            b_morale: Morale(1.0),
            flags: FlagsOwnership::empty(),
        }
    }

    pub fn from_copy(copy: &BattleStateCopy, map: &Map) -> Self {
        Self::new(
            copy.frame_i(),
            map.clone(),
            copy.soldiers().clone(),
            copy.vehicles().clone(),
            copy.soldier_on_board().clone(),
            copy.phase().clone(),
            copy.flags().clone(),
        )
    }

    pub fn resolve(&mut self) {
        // At start point, squads have not been defined. We must initialize it.
        self.update_squads();
        self.check_board_integrity()
            .expect("Error with board integrity imply programmatic error");
        self.initialize_vehicle_positions();
    }

    pub fn clean(&mut self, replaced_frame_i: Option<u64>) {
        let frame_i = replaced_frame_i.unwrap_or(self.frame_i);
        self.bullet_fires.retain(|b| !b.finished(frame_i));
        self.explosions.retain(|e| !e.finished(frame_i));
    }

    pub fn frame_i(&self) -> &u64 {
        &self.frame_i
    }

    pub fn map(&self) -> &Map {
        &self.map
    }

    pub fn visibilities(&self) -> &Visibilities {
        &self.visibilities
    }

    pub fn soldiers(&self) -> &Vec<Soldier> {
        &self.soldiers
    }

    pub fn soldier(&self, soldier_index: SoldierIndex) -> &Soldier {
        &self.soldiers[soldier_index.0]
    }

    pub fn soldier_mut(&mut self, soldier_index: SoldierIndex) -> &mut Soldier {
        &mut self.soldiers[soldier_index.0]
    }

    pub fn vehicle(&self, vehicle_index: VehicleIndex) -> &Vehicle {
        &self.vehicles[vehicle_index.0]
    }

    pub fn vehicles(&self) -> &Vec<Vehicle> {
        &self.vehicles
    }

    pub fn vehicle_mut(&mut self, vehicle_index: VehicleIndex) -> &mut Vehicle {
        &mut self.vehicles[vehicle_index.0]
    }

    pub fn squads(&self) -> &HashMap<SquadUuid, SquadComposition> {
        &self.squads
    }

    pub fn set_squads(&mut self, squads: HashMap<SquadUuid, SquadComposition>) {
        self.squads = squads;
    }

    pub fn all_orders(&self, side: &Side) -> Vec<(SquadUuid, &Order)> {
        let mut orders: Vec<(SquadUuid, &Order)> = vec![];

        for (squad_uuid, squad_composition) in &self.squads {
            if side != &Side::All && self.squad_side(squad_uuid) != side {
                continue;
            }

            let squad_leader = self.soldier(squad_composition.leader());
            orders.push((*squad_uuid, squad_leader.order()));
        }

        orders
    }

    pub fn squad_side(&self, squad_uuid: &SquadUuid) -> &Side {
        let composition = self.squad(*squad_uuid);
        let squad_leader = self.soldier(composition.leader());
        squad_leader.side()
    }

    pub fn squad(&self, squad_uuid: SquadUuid) -> &SquadComposition {
        self.squads
            .get(&squad_uuid)
            .expect("Game shared_state should never own inconsistent squad index")
    }

    pub fn bullet_fires(&self) -> &Vec<BulletFire> {
        self.bullet_fires.as_ref()
    }

    pub fn explosions(&self) -> &Vec<Explosion> {
        self.explosions.as_ref()
    }

    pub fn soldier_on_board(&self) -> &SoldiersOnBoard {
        &self.soldier_on_board
    }

    pub fn soldier_board(&self, soldier_index: SoldierIndex) -> Option<&SoldierBoard> {
        self.soldier_on_board.get(&soldier_index)
    }

    pub fn soldier_vehicle(&self, soldier_index: SoldierIndex) -> Option<VehicleIndex> {
        if let Some(soldier_board) = self.soldier_board(soldier_index) {
            return Some(soldier_board.0);
        }

        None
    }

    pub fn squad_path_mode_and_direction(
        &self,
        squad_id: SquadUuid,
    ) -> (PathMode, Option<Direction>) {
        let squad_leader_index = self.squad(squad_id).leader();
        if let Some(vehicle_index) = self.soldier_vehicle(squad_leader_index) {
            let vehicle = self.vehicle(vehicle_index);
            (
                PathMode::Drive(*VehicleGraphicInfos::from_type(vehicle.type_()).size()),
                Some(Direction::from_angle(vehicle.chassis_orientation())),
            )
        } else {
            (PathMode::Walk, None)
        }
    }

    pub fn vehicle_board(&self) -> &VehicleBoard {
        &self.vehicle_board
    }

    pub fn react(&mut self, state_message: &BattleStateMessage, frame_i: u64) -> Vec<SideEffect> {
        match state_message {
            BattleStateMessage::IncrementFrameI => self.frame_i += 1,
            BattleStateMessage::Soldier(soldier_index, soldier_message) => {
                return self.react_soldier_message(soldier_index, soldier_message);
            }
            BattleStateMessage::Vehicle(vehicle_index, vehicle_message) => {
                return self.react_vehicle_message(vehicle_index, vehicle_message);
            }
            BattleStateMessage::PushBulletFire(bullet_fire) => {
                let mut bullet_fire = bullet_fire.clone();
                bullet_fire.init(frame_i + 1);
                self.bullet_fires.push(bullet_fire)
            }
            BattleStateMessage::PushExplosion(explosion) => {
                let mut explosion = explosion.clone();
                explosion.init(frame_i + 1);
                self.explosions.push(explosion)
            }
            BattleStateMessage::SetVisibilities(visibilities) => {
                self.visibilities.set(visibilities.clone())
            }
            BattleStateMessage::SetPhase(phase) => self.phase = phase.clone(),
            BattleStateMessage::SetAConnected(value) => self.a_connected = *value,
            BattleStateMessage::SetBConnected(value) => self.b_connected = *value,
            BattleStateMessage::SetAReady(value) => self.a_ready = *value,
            BattleStateMessage::SetBReady(value) => self.b_ready = *value,
            BattleStateMessage::SetFlagsOwnership(flags) => self.flags = flags.clone(),
            BattleStateMessage::SetAMorale(morale) => self.a_morale = morale.clone(),
            BattleStateMessage::SetBMorale(morale) => self.b_morale = morale.clone(),
            BattleStateMessage::SetSquadLeader(squad_uuid, soldier_index) => {
                *self
                    .squads
                    .get_mut(squad_uuid)
                    .expect("Squad indexes must be consistent")
                    .leader_mut() = *soldier_index
            }
        };

        vec![]
    }

    pub fn inject(&mut self, deployment: &Deployment) {
        for soldier_deployment in deployment.soldiers() {
            self.soldiers.push(Soldier::from(soldier_deployment))
        }
        for vehicle_deployment in deployment.vehicles() {
            self.vehicles.push(Vehicle::from(vehicle_deployment))
        }
        self.soldier_on_board = deployment.boards().clone();
        self.resolve();
    }

    pub fn debug_lines(&self) -> Vec<(String, String)> {
        vec![
            (
                "Soldiers (len)".to_string(),
                self.soldiers.len().to_string(),
            ),
            ("Squads (len)".to_string(), self.squads.len().to_string()),
            (
                "Vehicles (len)".to_string(),
                self.vehicles.len().to_string(),
            ),
        ]
    }

    pub fn copy(&self) -> BattleStateCopy {
        BattleStateCopy::new(
            self.frame_i,
            self.soldiers.clone(),
            self.vehicles.clone(),
            self.soldier_on_board.clone(),
            self.phase.clone(),
            self.flags.clone(),
        )
    }

    pub fn phase(&self) -> &Phase {
        &self.phase
    }

    pub fn phase_mut(&mut self) -> &mut Phase {
        &mut self.phase
    }

    pub fn set_phase(&mut self, phase: Phase) {
        self.phase = phase;
    }

    pub fn a_connected(&self) -> bool {
        self.a_connected
    }

    pub fn b_connected(&self) -> bool {
        self.b_connected
    }

    pub fn a_ready(&self) -> bool {
        self.a_ready
    }

    pub fn b_ready(&self) -> bool {
        self.b_ready
    }

    pub fn ready(&self, side: &Side) -> bool {
        match side {
            Side::A => self.a_ready,
            Side::B => self.b_ready,
            Side::All => panic!("Never call ready for Side::All"),
        }
    }

    pub fn update_flags_from_control(&mut self, a_control: MapControl, b_control: MapControl) {
        self.flags = FlagsOwnership::from_control(&self.map, &a_control, &b_control);
    }

    pub fn flags(&self) -> &FlagsOwnership {
        &self.flags
    }

    pub fn there_is_side_soldier_in(&self, side: &Side, shape: WorldShape) -> bool {
        self.soldiers
            .iter()
            .filter(|s| s.side() == side)
            .filter(|s| s.can_take_flag())
            .any(|s| shape.contains(&s.world_point()))
    }

    pub fn a_morale(&self) -> &Morale {
        &self.a_morale
    }

    pub fn b_morale(&self) -> &Morale {
        &self.b_morale
    }
}

#[derive(Debug)]
pub enum BattleStateError {
    BoardIntegrity(String),
}
