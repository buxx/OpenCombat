use derive_more::Constructor;
use oc_mod::Mod;

use crate::{
    deployment::{
        Deployment,
        individual::Individual,
        magazine::Magazine,
        squad::Squad,
        vehicle::{CrewMember, Shells, Vehicle},
        weapon::Weapons,
    },
    generator::profile::Profile,
};

pub mod profile;

#[derive(Debug, Constructor)]
pub struct Generator {}

impl Generator {
    pub fn generate(&self, mod_: &Mod, profile: &Profile) -> Result<Deployment, anyhow::Error> {
        let mut deployment = Deployment::default();

        for squad in &profile.squads {
            for _ in 0..squad.count {
                deployment.squads.push(self.squad(mod_, squad)?);
            }
        }

        Ok(deployment)
    }

    fn squad(&self, mod_: &Mod, squad: &profile::Squad) -> Result<Squad, anyhow::Error> {
        let mut squad_ = Squad::new();
        squad_.label = squad.label.clone();
        squad_.illustration = squad.illustration.clone();

        for individual in &squad.individuals {
            for _ in 0..individual.count {
                squad_.individuals.push(self.individual(mod_, individual)?);
            }
        }

        if let Some(name) = &squad.vehicle {
            squad_.vehicle = Some(self.vehicle(mod_, name, squad, &squad_.individuals)?);
        }

        Ok(squad_)
    }

    fn individual(
        &self,
        mod_: &Mod,
        individual: &profile::Individual,
    ) -> Result<Individual, anyhow::Error> {
        let mut individual_ = Individual::new();
        individual_.illustration = individual.illustration.clone();
        individual_.xp = random_xp(individual.xp);
        individual_.weapons = self.weapons(mod_, &individual.weapons)?;

        for magazine in &individual.magazines {
            for _ in 0..magazine.count {
                individual_.magazines.push(self.magazine(mod_, magazine)?);
            }
        }

        Ok(individual_)
    }

    fn weapons(&self, mod_: &Mod, weapons: &profile::Weapons) -> Result<Weapons, anyhow::Error> {
        let mut weapons_ = Weapons::default();

        if let Some(weapon) = &weapons.main {
            let _ = mod_.weapon_from_name(weapon)?;
            weapons_.main = Some(weapon.clone());
        }

        for grenade in &weapons.grenades {
            let _ = mod_.grenade_from_name(grenade)?;
            weapons_.grenades.push(grenade.clone());
        }

        Ok(weapons_)
    }

    fn magazine(
        &self,
        mod_: &Mod,
        magazine: &profile::Magazine,
    ) -> Result<Magazine, anyhow::Error> {
        let magazine_ = Magazine {
            name: magazine.name.clone(),
            ammunition: magazine.ammunition.clone(),
        };
        magazine_.ammunition(mod_, mod_.magazine_from_name(&magazine.name)?)?;
        Ok(magazine_)
    }

    fn vehicle(
        &self,
        mod_: &Mod,
        name: &str,
        squad: &profile::Squad,
        individuals: &[Individual],
    ) -> Result<Vehicle, anyhow::Error> {
        let vehicle = mod_.vehicle_from_name(name)?;
        let mut vehicle_ = Vehicle::new(name.to_string());
        vehicle_.crew = vehicle
            .crew
            .iter()
            .zip(individuals)
            .map(|(role, individual)| CrewMember {
                role: *role,
                individual: individual.uuid,
            })
            .collect();

        for shells in &squad.shells {
            let _ = mod_.shell_from_name(&shells.name)?;
            vehicle_.shells.push(Shells {
                name: shells.name.clone(),
                count: shells.count,
            });
        }

        Ok(vehicle_)
    }
}

/// `base` plus or minus a random value in `0..=range`, clamped to `u8`
fn random_xp([base, range]: [u8; 2]) -> u8 {
    let (base, range) = (base as i16, range as i16);
    (base + fastrand::i16(-range..=range)).clamp(0, 255) as u8
}
