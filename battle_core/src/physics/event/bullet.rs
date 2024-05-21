use crate::{
    game::weapon::{Ammunition, GunFireSoundType, Shot},
    types::{Precision, SoldierIndex, WorldPoint},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulletFire {
    start: u64,
    end: u64,
    from: WorldPoint,
    to: WorldPoint,
    target: Option<(SoldierIndex, Precision)>,
    ammunition: Ammunition,
    gun_fire_sound_type: Option<GunFireSoundType>,
    // FIXME BS NOW : remove ?
    shot: Shot,
}

impl BulletFire {
    pub fn new(
        // Used as offset (machine gun)
        start: u64,
        from: WorldPoint,
        to: WorldPoint,
        target: Option<(SoldierIndex, Precision)>,
        ammunition: Ammunition,
        gun_fire_sound_type: Option<GunFireSoundType>,
        shot: Shot,
    ) -> Self {
        Self {
            start,
            end: 0,
            from,
            to,
            target,
            ammunition,
            gun_fire_sound_type,
            shot,
        }
    }

    pub fn init(&mut self, start_frame_i: u64) {
        self.start += start_frame_i;
        // FIXME: "2" configurable ?
        self.end = self.start + 2;
    }

    pub fn point(&self) -> &WorldPoint {
        &self.to
    }

    pub fn ammunition(&self) -> &Ammunition {
        &self.ammunition
    }

    pub fn finished(&self, frame_i: u64) -> bool {
        frame_i >= self.end
    }

    pub fn effective(&self, frame_i: u64) -> bool {
        self.start == frame_i
    }

    pub fn from(&self) -> &WorldPoint {
        &self.from
    }

    pub fn to(&self) -> &WorldPoint {
        &self.to
    }

    pub fn start(&self) -> u64 {
        self.start
    }

    pub fn end(&self) -> u64 {
        self.end
    }

    pub fn gun_fire_sound_type(&self) -> &Option<GunFireSoundType> {
        &self.gun_fire_sound_type
    }

    pub fn shots(&self) -> &Shot {
        &self.shot
    }
}
