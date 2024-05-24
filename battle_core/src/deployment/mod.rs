use std::{collections::HashMap, fs, io, path::PathBuf};

use oc_core::game::{soldier::SoldierType, squad::SquadType};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    behavior::Behavior,
    entity::{
        soldier::Soldier,
        vehicle::{Vehicle, VehicleType},
    },
    game::{
        weapon::{Magazine, Weapon},
        Side,
    },
    order::Order,
    state::battle::BattleState,
    types::{SoldierIndex, SoldiersOnBoard, SquadUuid, VehicleIndex, WorldPoint},
};

pub type SquadTypes = HashMap<SquadUuid, SquadType>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Deployment {
    soldiers: Vec<SoldierDeployment>,
    vehicles: Vec<VehicleDeployment>,
    boards: SoldiersOnBoard,
    squad_types: SquadTypes,
}

impl Deployment {
    pub fn new(
        soldiers: Vec<SoldierDeployment>,
        vehicles: Vec<VehicleDeployment>,
        boards: SoldiersOnBoard,
        squad_types: SquadTypes,
    ) -> Self {
        Self {
            soldiers,
            vehicles,
            boards,
            squad_types,
        }
    }

    pub fn empty() -> Self {
        Self {
            soldiers: vec![],
            vehicles: vec![],
            boards: HashMap::new(),
            squad_types: HashMap::new(),
        }
    }

    pub fn from_battle_state(battle_state: &BattleState) -> Self {
        let soldiers: Vec<SoldierDeployment> = battle_state
            .soldiers()
            .iter()
            .map(SoldierDeployment::from)
            .collect();
        let vehicles: Vec<VehicleDeployment> = battle_state
            .vehicles()
            .iter()
            .map(VehicleDeployment::from)
            .collect();
        let squad_types: SquadTypes = battle_state
            .squads()
            .iter()
            .map(|s| (*s.0, s.1.type_().clone()))
            .collect();

        Self {
            soldiers,
            vehicles,
            boards: battle_state.soldier_on_board().clone(),
            squad_types,
        }
    }

    pub fn soldiers(&self) -> &[SoldierDeployment] {
        self.soldiers.as_ref()
    }

    pub fn vehicles(&self) -> &[VehicleDeployment] {
        self.vehicles.as_ref()
    }

    pub fn boards(&self) -> &SoldiersOnBoard {
        &self.boards
    }

    pub fn squad_types(&self) -> &SquadTypes {
        &self.squad_types
    }
}

impl
    From<(
        Vec<SoldierDeployment>,
        Vec<VehicleDeployment>,
        SoldiersOnBoard,
        SquadTypes,
    )> for Deployment
{
    fn from(
        value: (
            Vec<SoldierDeployment>,
            Vec<VehicleDeployment>,
            SoldiersOnBoard,
            SquadTypes,
        ),
    ) -> Self {
        Self {
            soldiers: value.0,
            vehicles: value.1,
            boards: value.2,
            squad_types: value.3,
        }
    }
}

pub struct DeploymentReader;

impl DeploymentReader {
    pub fn from_file(path: &PathBuf) -> Result<Deployment, DeploymentReaderError> {
        let deployment: Deployment = serde_json::from_str(&fs::read_to_string(path)?)?;
        Ok(deployment)
    }
}

#[derive(Error, Debug)]
pub enum DeploymentReaderError {
    #[error("Error during file read")]
    Read(#[from] io::Error),
    #[error("Data format error")]
    Format(#[from] serde_json::Error),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SoldierDeployment {
    uuid: SoldierIndex,
    type_: SoldierType,
    side: Side,
    world_point: WorldPoint,
    squad_uuid: SquadUuid,
    main_weapon: Option<Weapon>,
    magazines: Vec<Magazine>,
    order: Order,
    behavior: Behavior,
}

impl SoldierDeployment {
    pub fn new(
        uuid: SoldierIndex,
        type_: SoldierType,
        side: Side,
        world_point: WorldPoint,
        squad_uuid: SquadUuid,
        main_weapon: Option<Weapon>,
        magazines: Vec<Magazine>,
        order: Order,
        behavior: Behavior,
    ) -> Self {
        Self {
            uuid,
            type_,
            side,
            world_point,
            squad_uuid,
            main_weapon,
            magazines,
            order,
            behavior,
        }
    }

    pub fn uuid(&self) -> SoldierIndex {
        self.uuid
    }

    pub fn side(&self) -> Side {
        self.side
    }

    pub fn world_point(&self) -> WorldPoint {
        self.world_point
    }

    pub fn squad_uuid(&self) -> SquadUuid {
        self.squad_uuid
    }

    pub fn main_weapon(&self) -> Option<&Weapon> {
        self.main_weapon.as_ref()
    }

    pub fn magazines(&self) -> &[Magazine] {
        self.magazines.as_ref()
    }

    pub fn order(&self) -> &Order {
        &self.order
    }

    pub fn behavior(&self) -> &Behavior {
        &self.behavior
    }

    pub fn type_(&self) -> &SoldierType {
        &self.type_
    }
}

impl From<&Soldier> for SoldierDeployment {
    fn from(soldier: &Soldier) -> Self {
        Self {
            uuid: soldier.uuid(),
            type_: *soldier.type_(),
            side: *soldier.side(),
            world_point: soldier.world_point(),
            squad_uuid: soldier.squad_uuid(),
            main_weapon: soldier.main_weapon().clone(),
            magazines: soldier.magazines().clone(),
            order: soldier.order().clone(),
            behavior: soldier.behavior().clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VehicleDeployment {
    uuid: VehicleIndex,
    type_: VehicleType,
    world_point: WorldPoint,
}

impl VehicleDeployment {
    pub fn new(uuid: VehicleIndex, type_: VehicleType, world_point: WorldPoint) -> Self {
        Self {
            uuid,
            type_,
            world_point,
        }
    }

    pub fn uuid(&self) -> VehicleIndex {
        self.uuid
    }

    pub fn type_(&self) -> &VehicleType {
        &self.type_
    }

    pub fn world_point(&self) -> WorldPoint {
        self.world_point
    }
}

impl From<&Vehicle> for VehicleDeployment {
    fn from(vehicle: &Vehicle) -> Self {
        Self {
            uuid: *vehicle.uuid(),
            type_: vehicle.type_().clone(),
            world_point: vehicle.world_point(),
        }
    }
}
