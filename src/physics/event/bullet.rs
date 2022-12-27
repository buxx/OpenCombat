use crate::{
    game::weapon::Weapon,
    message::{Message, SharedStateMessage},
    types::{Precision, SoldierIndex, WorldPoint},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulletFire {
    new: bool,
    start: u64,
    end: u64,
    from: WorldPoint,
    to: WorldPoint,
    target: Option<(SoldierIndex, Precision)>,
    weapon: Weapon,
}

impl BulletFire {
    pub fn new(
        frame_i: u64,
        from: WorldPoint,
        to: WorldPoint,
        target: Option<(SoldierIndex, Precision)>,
        weapon: Weapon,
    ) -> Self {
        Self {
            new: true,
            start: frame_i,
            end: frame_i + 5,
            from,
            to,
            target,
            weapon,
        }
    }

    pub fn tick(&mut self, frame_i: u64) -> bool {
        self.new = false;
        frame_i >= self.end
    }

    pub fn messages(&self, _frame_i: u64) -> Vec<Message> {
        let mut messages = vec![];

        if self.new {
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
