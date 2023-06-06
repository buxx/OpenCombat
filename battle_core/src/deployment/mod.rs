use std::{fs, io, path::PathBuf};

use serde_derive::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    entity::{
        soldier::Soldier,
        vehicle::{Vehicle, VehicleType},
    },
    game::{
        weapon::{Magazine, Weapon},
        Side,
    },
    state::battle::BattleState,
    types::{SoldierIndex, SoldiersOnBoard, SquadUuid, VehicleIndex, WorldPoint},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Deployment {
    soldiers: Vec<SoldierDeployment>,
    vehicles: Vec<VehicleDeployment>,
    boards: SoldiersOnBoard,
}

impl Deployment {
    pub fn from_battle_state(battle_state: &BattleState) -> Self {
        let soldiers: Vec<SoldierDeployment> = battle_state
            .soldiers()
            .iter()
            .map(|s| SoldierDeployment::from(s))
            .collect();
        let vehicles: Vec<VehicleDeployment> = battle_state
            .vehicles()
            .iter()
            .map(|v| VehicleDeployment::from(v))
            .collect();

        Self {
            soldiers,
            vehicles,
            boards: battle_state.soldier_on_board().clone(),
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
}

impl
    From<(
        Vec<SoldierDeployment>,
        Vec<VehicleDeployment>,
        SoldiersOnBoard,
    )> for Deployment
{
    fn from(
        value: (
            Vec<SoldierDeployment>,
            Vec<VehicleDeployment>,
            SoldiersOnBoard,
        ),
    ) -> Self {
        Self {
            soldiers: value.0,
            vehicles: value.1,
            boards: value.2,
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
    side: Side,
    world_point: WorldPoint,
    squad_uuid: SquadUuid,
    main_weapon: Option<Weapon>,
    magazines: Vec<Magazine>,
}

impl SoldierDeployment {
    pub fn new(
        uuid: SoldierIndex,
        side: Side,
        world_point: WorldPoint,
        squad_uuid: SquadUuid,
        main_weapon: Option<Weapon>,
        magazines: Vec<Magazine>,
    ) -> Self {
        Self {
            uuid,
            side,
            world_point,
            squad_uuid,
            main_weapon,
            magazines,
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
}

impl From<&Soldier> for SoldierDeployment {
    fn from(soldier: &Soldier) -> Self {
        Self {
            uuid: soldier.uuid().clone(),
            side: soldier.get_side().clone(),
            world_point: soldier.get_world_point().clone(),
            squad_uuid: soldier.squad_uuid().clone(),
            main_weapon: soldier.main_weapon().clone(),
            magazines: soldier.magazines().clone(),
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
            uuid: vehicle.uuid().clone(),
            type_: vehicle.get_type().clone(),
            world_point: vehicle.get_world_point().clone(),
        }
    }
}
