use crate::{
    config::TARGET_FPS,
    game::explosive::Type as ExplosiveType,
    message::{GraphicsMessage, Message, SharedStateMessage},
    types::WorldPoint,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Explosion {
    start: u64,
    end: u64,
    point: WorldPoint,
    explosive_type: ExplosiveType,
}

impl Explosion {
    pub fn new(point: WorldPoint, type_: ExplosiveType) -> Self {
        Self {
            start: 0,
            end: 0,
            point,
            explosive_type: type_,
        }
    }

    pub fn init(&mut self, start_frame_i: u64) {
        self.start = start_frame_i;
        self.end =
            start_frame_i + (self.explosive_type.sprite().duration() as u32 * TARGET_FPS) as u64;
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
            for sound in self.explosive_type.sounds() {
                messages.push(Message::SharedState(SharedStateMessage::PushSoundToPlay(
                    sound.clone(),
                )));
            }

            messages.push(Message::Graphics(GraphicsMessage::PushExplosionAnimation(
                self.point.clone(),
                self.explosive_type.clone(),
            )));
        }

        if frame_i >= self.end {
            // TODO : Remove by self.point can remove other explosions. Find better methodology
            messages.push(Message::Graphics(
                GraphicsMessage::RemoveExplosionAnimation(self.point.clone()),
            ))
        }

        messages
    }
}
