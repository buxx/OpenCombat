use std::collections::HashMap;

use ggez::GameResult;

use crate::{
    entity::{soldier::Soldier, vehicle::Vehicle},
    game::Side,
    graphics::vehicle::VehicleGraphicInfos,
    message::*,
    order::Order,
    physics::{
        effect::Effect,
        path::{Direction, PathMode},
    },
    sync::StateCopy,
    types::*,
    utils::vehicle_board_from_soldiers_on_board,
};

use super::SideEffect;

pub struct SharedState {
    /// Used to ignore server shared_state modifications since shared state not received from server
    initialized: bool,
    /// The soldiers in the world
    soldiers: Vec<Soldier>,
    /// The vehicles in the world
    vehicles: Vec<Vehicle>,
    /// Vehicle on board information
    soldier_on_board: SoldiersOnBoard,
    vehicle_board: VehicleBoard,
    /// Squad organizations, must be updated when squad leader changes.
    squads: HashMap<SquadUuid, SquadComposition>,
    /// Players orders. Squad leaders will pick from them theirs behaviors.
    command_orders: HashMap<SquadUuid, Order>,
    /// Squad leader orders. Squad members will pick from them theirs behaviors.
    squad_orders: HashMap<SoldierIndex, Order>,
    /// Physics effects to deal with
    physics_effects: Vec<Effect>,
}

impl SharedState {
    pub fn new(
        soldiers: Vec<Soldier>,
        vehicles: Vec<Vehicle>,
        soldier_on_board: SoldiersOnBoard,
    ) -> Self {
        let vehicle_board = vehicle_board_from_soldiers_on_board(&soldier_on_board);
        Self {
            initialized: false,
            soldiers,
            vehicles,
            soldier_on_board: soldier_on_board,
            vehicle_board,
            squads: HashMap::new(),
            command_orders: HashMap::new(),
            squad_orders: HashMap::new(),
            physics_effects: vec![],
        }
    }

    pub fn init(&mut self) -> GameResult {
        // At start point, squads have not been defined. We must initialize it.
        self.update_squads();
        self.check_board_integrity()?;
        self.initialize_vehicle_positions();
        self.initialized = true;
        Ok(())
    }

    pub fn init_from_copy(&mut self, state_copy: StateCopy) {
        self.soldiers = vec![];
        self.vehicles = vec![];

        for soldier in state_copy.soldiers() {
            self.soldiers.push(soldier.clone());
        }

        for vehicle in state_copy.vehicles() {
            self.vehicles.push(vehicle.clone());
        }

        // TODO
        self.init().unwrap();
    }

    pub fn initialized(&self) -> bool {
        self.initialized
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

    pub fn command_orders(&self) -> &HashMap<SquadUuid, Order> {
        &self.command_orders
    }

    pub fn squad_orders(&self) -> &HashMap<SoldierIndex, Order> {
        &self.squad_orders
    }

    pub fn all_orders(&self, side: &Side) -> Vec<(SquadUuid, Order)> {
        let mut orders: Vec<(SquadUuid, Order)> = vec![];

        for (squad_uuid, squad_composition) in &self.squads {
            if side != &Side::All && self.squad_side(squad_uuid) != side {
                continue;
            }

            let squad_leader = self.soldier(squad_composition.leader());
            if let Some(order) = squad_leader.get_behavior().to_order() {
                orders.push((*squad_uuid, order));
            }
        }

        for (squad_id, order) in &self.command_orders {
            orders.push((*squad_id, order.clone()));
        }

        orders
    }

    pub fn squad_side(&self, squad_uuid: &SquadUuid) -> &Side {
        let composition = self.squad(*squad_uuid);
        let squad_leader = self.soldier(composition.leader());
        squad_leader.get_side()
    }

    pub fn squad(&self, squad_uuid: SquadUuid) -> &SquadComposition {
        self.squads
            .get(&squad_uuid)
            .expect("Game shared_state should never own inconsistent squad index")
    }

    pub fn _soldier_on_board(&self) -> &SoldiersOnBoard {
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
                PathMode::Drive(*VehicleGraphicInfos::from_type(vehicle.get_type()).size()),
                Some(Direction::from_angle(vehicle.get_chassis_orientation())),
            )
        } else {
            (PathMode::Walk, None)
        }
    }

    pub fn vehicle_board(&self) -> &VehicleBoard {
        &self.vehicle_board
    }

    pub fn physics_effects(&self) -> &Vec<Effect> {
        &self.physics_effects
    }

    pub fn physics_effects_mut(&mut self) -> &mut Vec<Effect> {
        &mut self.physics_effects
    }

    pub fn react(&mut self, state_message: crate::message::SharedStateMessage) -> Vec<SideEffect> {
        match state_message {
            SharedStateMessage::Soldier(soldier_index, soldier_message) => {
                return self.react_soldier_message(soldier_index, soldier_message);
            }
            SharedStateMessage::Vehicle(vehicle_index, vehicle_message) => {
                return self.react_vehicle_message(vehicle_index, vehicle_message);
            }
            SharedStateMessage::PushCommandOrder(squad_uuid, order) => {
                self.command_orders.insert(squad_uuid, order);
            }
            SharedStateMessage::PushSquadOrder(soldier_index, order) => {
                self.squad_orders.insert(soldier_index, order);
            }
            SharedStateMessage::RemoveCommandOder(squad_uuid) => {
                self.command_orders
                    .remove(&squad_uuid)
                    .expect("Game shared_state should never own inconsistent orders index");
            }
            SharedStateMessage::RemoveSquadOder(soldier_index) => {
                self.squad_orders
                    .remove(&soldier_index)
                    .expect("Game shared_state should never own inconsistent orders index");
            }
            SharedStateMessage::PushSoundToPlay(sound) => {
                //
                return vec![SideEffect::PlaySound(sound)];
            }
            SharedStateMessage::PushPhysicsEffect(effect) => {
                //
                self.physics_effects.push(effect)
            }
        };

        vec![]
    }
}
