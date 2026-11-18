use derive_more::Constructor;
use oc_mod::Mod;

use crate::{
    deployment::{
        Deployment, individual::Individual, magazine::Magazine, squad::Squad, weapon::Weapons,
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
                let mut squad_ = Squad::new();
                squad_.label = squad.label.clone();
                squad_.illustration = squad.illustration.clone();

                for individual in &squad.individuals {
                    for _ in 0..individual.count {
                        let mut individual_ = Individual::new();
                        individual_.illustration = individual.illustration.clone();
                        individual_.xp = random_xp(individual.xp);

                        individual_.weapons = Weapons::default();
                        if let Some(weapon) = &individual.weapons.main {
                            let _ = mod_.weapon_from_name(weapon)?;
                            individual_.weapons.main = Some(weapon.clone());
                        }

                        for grenade in &individual.weapons.grenades {
                            let _ = mod_.grenade_from_name(grenade)?;
                            individual_.weapons.grenades.push(grenade.clone());
                        }

                        for magazine in &individual.magazines {
                            for _ in 0..magazine.count {
                                let _ = mod_.magazine_from_name(&magazine.name)?;
                                let magazine_ = Magazine {
                                    name: magazine.name.clone(),
                                };
                                individual_.magazines.push(magazine_);
                            }
                        }

                        squad_.individuals.push(individual_);
                    }
                }

                deployment.squads.push(squad_);
            }
        }

        Ok(deployment)
    }
}

/// `base` plus or minus a random value in `0..=range`, clamped to `u8`
fn random_xp([base, range]: [u8; 2]) -> u8 {
    let (base, range) = (base as i16, range as i16);
    (base + fastrand::i16(-range..=range)).clamp(0, 255) as u8
}
