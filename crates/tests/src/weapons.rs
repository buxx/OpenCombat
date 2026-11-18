use bon::Builder;
use oc_individual::{Weapon, Weapons};
use oc_mod::{
    Mod, ammunition::AmmunitionIndex, grenade::GrenadeIndex, magazine::MagazineIndex,
    weapons::WeaponIndex,
};

#[derive(Debug, Builder)]
pub struct TestWeapons {
    primary: Option<Weapon>,
    #[builder(default)]
    grenades: Vec<GrenadeIndex>,
}

impl TestWeapons {
    pub fn make(self) -> Weapons {
        Weapons {
            primary: self.primary,
            grenades: self.grenades,
        }
    }
}

#[derive(Debug, Builder)]
pub struct TestWeapon {
    i: WeaponIndex,
    filled: Option<(MagazineIndex, AmmunitionIndex)>,
    filled_count: u16,
}

impl TestWeapon {
    pub fn filled(mod_: &Mod, name: &str) -> TestWeapon {
        let weapon = mod_.weapons.iter().find(|w| w.name() == name).unwrap();
        let magazine = weapon
            .magazines()
            .iter()
            .find_map(|m| mod_.magazines.iter().find(|m_| m.name() == m_.name()))
            .unwrap();
        let ammunition = weapon.ammunitions().first().unwrap();

        TestWeapon {
            i: weapon.index(),
            filled: Some((magazine.index(), ammunition.index())),
            filled_count: magazine.capacity(),
        }
    }

    pub fn not_filled(mod_: &Mod, name: &str) -> TestWeapon {
        let weapon = mod_.weapons.iter().find(|w| w.name() == name).unwrap();
        TestWeapon {
            i: weapon.index(),
            filled: None,
            filled_count: 0,
        }
    }

    pub fn make(self) -> Weapon {
        Weapon {
            i: self.i,
            filled: self.filled,
            filled_count: self.filled_count,
        }
    }
}
