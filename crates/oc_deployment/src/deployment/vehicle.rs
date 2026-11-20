use oc_mod::vehicle::CrewRole;
use oc_root::uuids::IndividualUuid;
use serde::{Deserialize, Serialize};

/// Vehicle operated by a squad
#[derive(Debug, Deserialize, Serialize)]
pub struct Vehicle {
    /// Mod vehicle name (see mod `vehicles.ron`)
    pub name: String,
    /// Squad individuals manning the vehicle
    #[serde(default)]
    pub crew: Vec<CrewMember>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CrewMember {
    pub role: CrewRole,
    /// Must be an individual of the squad owning the vehicle
    pub individual: IndividualUuid,
}

impl Vehicle {
    pub fn new(name: String) -> Self {
        Self { name, crew: vec![] }
    }

    pub fn validate(&self, mod_: &oc_mod::Mod, squad: &super::squad::Squad) -> Result<(), Error> {
        let vehicle = mod_
            .vehicle_from_name(&self.name)
            .map_err(|_| Error::UnknownVehicle(self.name.clone()))?;

        for (i, member) in self.crew.iter().enumerate() {
            if !vehicle.crew.contains(&member.role) {
                return Err(Error::UnknownRole(self.name.clone(), member.role));
            }
            if self.crew[..i].iter().any(|m| m.role == member.role) {
                return Err(Error::DuplicateRole(self.name.clone(), member.role));
            }
            if self.crew[..i]
                .iter()
                .any(|m| m.individual == member.individual)
            {
                return Err(Error::DuplicateIndividual(member.individual));
            }
            if !squad
                .individuals
                .iter()
                .any(|i| i.uuid == member.individual)
            {
                return Err(Error::NotInSquad(member.individual, squad.uuid));
            }
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unknown vehicle name: {0}")]
    UnknownVehicle(String),
    #[error("Vehicle {0} has no crew role {1:?}")]
    UnknownRole(String, CrewRole),
    #[error("Vehicle {0} crew role {1:?} assigned more than once")]
    DuplicateRole(String, CrewRole),
    #[error("Individual {0} assigned to more than one crew role")]
    DuplicateIndividual(IndividualUuid),
    #[error("Crew individual {0} is not part of squad {1}")]
    NotInSquad(IndividualUuid, oc_root::uuids::SquadUuid),
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::deployment::{individual::Individual, squad::Squad};

    fn mod_() -> oc_mod::Mod {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../mods/tests1");
        oc_mod::Mod::load(&path, None).unwrap()
    }

    fn squad(count: usize) -> Squad {
        let mut squad = Squad::new();
        squad.individuals = (0..count).map(|_| Individual::new()).collect();
        squad
    }

    fn member(role: CrewRole, squad: &Squad, i: usize) -> CrewMember {
        CrewMember {
            role,
            individual: squad.individuals[i].uuid,
        }
    }

    #[test]
    fn validate() {
        let mod_ = mod_();
        let squad = squad(3);
        let mut vehicle = Vehicle::new("Tank1".to_string());
        vehicle.crew = vec![
            member(CrewRole::Commander, &squad, 0),
            member(CrewRole::Driver, &squad, 1),
        ];
        vehicle.validate(&mod_, &squad).unwrap();

        // Unknown vehicle
        let unknown = Vehicle::new("Unknown".to_string());
        assert!(matches!(
            unknown.validate(&mod_, &squad),
            Err(Error::UnknownVehicle(_))
        ));

        // Role not declared by mod vehicle
        vehicle.crew = vec![member(CrewRole::Loader, &squad, 0)];
        assert!(matches!(
            vehicle.validate(&mod_, &squad),
            Err(Error::UnknownRole(..))
        ));

        // Same role twice
        vehicle.crew = vec![
            member(CrewRole::Driver, &squad, 0),
            member(CrewRole::Driver, &squad, 1),
        ];
        assert!(matches!(
            vehicle.validate(&mod_, &squad),
            Err(Error::DuplicateRole(..))
        ));

        // Same individual twice
        vehicle.crew = vec![
            member(CrewRole::Driver, &squad, 0),
            member(CrewRole::Gunner, &squad, 0),
        ];
        assert!(matches!(
            vehicle.validate(&mod_, &squad),
            Err(Error::DuplicateIndividual(_))
        ));

        // Individual from another squad
        vehicle.crew = vec![member(CrewRole::Driver, &super::tests::squad(1), 0)];
        assert!(matches!(
            vehicle.validate(&mod_, &squad),
            Err(Error::NotInSquad(..))
        ));
    }

    #[test]
    fn serde_roundtrip() {
        let squad = squad(1);
        let mut vehicle = Vehicle::new("Tank1".to_string());
        vehicle.crew = vec![member(CrewRole::Commander, &squad, 0)];
        let yaml = serde_yaml::to_string(&vehicle).unwrap();
        let back: Vehicle = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(back.name, "Tank1");
        assert_eq!(back.crew.len(), 1);
        assert_eq!(back.crew[0].role, CrewRole::Commander);
        assert_eq!(back.crew[0].individual, squad.individuals[0].uuid);
    }
}
