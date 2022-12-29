use std::collections::HashMap;

use ggez::GameResult;

use crate::{
    audio::Sound,
    entity::{soldier::Soldier, vehicle::Vehicle},
    game::Side,
    message::*,
    order::Order,
    physics::Physics,
    sync::StateCopy,
    types::*,
    utils::vehicle_board_from_soldiers_on_board,
};

use super::{local::LocalState, SideEffect};

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
    /// Vector of emitted physic event.
    physics: Physics,
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
            physics: Physics::new(),
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

    pub fn vehicle_board(&self) -> &VehicleBoard {
        &self.vehicle_board
    }

    pub fn physics(&self) -> &Physics {
        &self.physics
    }

    pub fn physics_mut(&mut self) -> &mut Physics {
        &mut self.physics
    }

    pub fn react(
        &mut self,
        state_message: crate::message::SharedStateMessage,
        local_state: &LocalState,
    ) -> Vec<SideEffect> {
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
            SharedStateMessage::PushBulletFire(mut bullet_fire) => {
                bullet_fire.init(local_state.get_frame_i());
                self.physics.bullet_fires.push(bullet_fire)
            }
            SharedStateMessage::PushExplosion(mut explosion) => {
                explosion.init(local_state.get_frame_i());
                self.physics.explosions.push(explosion)
            }
            SharedStateMessage::PushSoundToPlay(sound) => {
                //
                return vec![SideEffect::PlaySound(sound)];
            }
        };

        vec![]
    }
}
