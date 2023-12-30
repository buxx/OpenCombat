use serde::{Deserialize, Serialize};

use strum_macros::Display;
use strum_macros::EnumIter;

#[derive(Debug, Hash, Copy, Serialize, Deserialize, Clone, EnumIter, Eq, PartialEq, Display)]
pub enum Sound {
    MauserRiffleFire1,
    MauserRiffleFire2,
    MauserRiffleFire3,
    MauserRiffleReload1,
    MauserRiffleReload2,
    MosinNagantFire1,
    MosinNagantFire2,
    MosinNagantFire3,
    MosinNagantFire4,
    MosinNagantFire5,
    MosinNagantReload1,
    MosinNagantReload2,
    MosinNagantReload3,
    MosinNagantReload4,
    CannonFire1,
    MaleScreaming1,
    MaleScreaming2,
    MaleScreaming3,
    MaleScreaming4,
    MaleScreaming5,
    MaleDie1,
    MaleDie2,
    MaleDie3,
    MaleDie4,
    MaleDie5,
    MaleDie6,
    MaleDie7,
    MaleDie8,
    MetalHit1,
    Bip1,
    Clac1,
    Clic1,
    DrumMultiHits,
    TrumpetLongHall,
    BulletMetalImpact1,
    BulletTrunkImpact1,
    BulletWallImpact1,
    BulletGroundImpact1,
}

impl Sound {
    pub fn file_path(&self) -> String {
        format!("/audio/{}.ogg", self)
    }
}
