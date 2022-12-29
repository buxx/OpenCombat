use crate::{
    config::TARGET_FPS,
    game::explosive::Type as ExplosiveType,
    message::{GraphicsMessage, Message, SharedStateMessage},
    types::WorldPoint,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Explosion {
    new: bool,
    start: u64,
    end: u64,
    point: WorldPoint,
    explosive_type: ExplosiveType,
}

impl Explosion {
    pub fn new(point: WorldPoint, type_: ExplosiveType) -> Self {
        Self {
            new: true,
            start: 0,
            end: 0,
            point,
            explosive_type: type_,
        }
    }

    pub fn init(&mut self, frame_i: u64) {
        self.start = frame_i;
        self.end = frame_i + (self.explosive_type.sprite().duration() as u32 * TARGET_FPS) as u64;
    }

    pub fn tick(&mut self, frame_i: u64) -> bool {
        self.new = false;
        frame_i >= self.end
    }

    pub fn messages(&self, frame_i: u64) -> Vec<Message> {
        let mut messages = vec![];

        if self.new {
            for sound in self.explosive_type.sounds() {
                messages.extend(vec![
                    Message::SharedState(SharedStateMessage::PushSoundToPlay(sound.clone())),
                    Message::Graphics(GraphicsMessage::PushExplosionAnimation(
                        self.point.clone(),
                        self.explosive_type.clone(),
                    )),
                ]);
            }
        }

        if frame_i >= self.end {
            // TODO : Remove by self.point can remove other explosions
            messages.push(Message::Graphics(
                GraphicsMessage::RemoveExplosionAnimation(self.point.clone()),
            ))
        }

        messages
    }
}
