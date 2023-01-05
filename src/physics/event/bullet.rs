use crate::{
    game::weapon::Weapon,
    message::{Message, SharedStateMessage},
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
    weapon: Weapon,
}

impl BulletFire {
    pub fn new(
        from: WorldPoint,
        to: WorldPoint,
        target: Option<(SoldierIndex, Precision)>,
        weapon: Weapon,
    ) -> Self {
        Self {
            start: 0,
            end: 0,
            from,
            to,
            target,
            weapon,
        }
    }

    pub fn init(&mut self, start_frame_i: u64) {
        self.start = start_frame_i;
        self.end = start_frame_i + 5;
    }

    pub fn point(&self) -> &WorldPoint {
        &self.to
    }

    pub fn weapon(&self) -> &Weapon {
        &&self.weapon
    }

    pub fn finished(&self, frame_i: u64) -> bool {
        frame_i >= self.end
    }

    pub fn effective(&self, frame_i: u64) -> bool {
        self.start == frame_i
    }

    pub fn fx(&self, frame_i: u64) -> Vec<Message> {
        let mut messages = vec![];

        if self.start == frame_i {
            for sound in self.weapon.fire_sounds() {
                messages.push(Message::SharedState(SharedStateMessage::PushSoundToPlay(
                    sound.clone(),
                )));
            }
        }

        messages
    }

    pub fn from(&self) -> &WorldPoint {
        &self.from
    }

    pub fn to(&self) -> &WorldPoint {
        &self.to
    }
}
