use crate::{
    game::weapon::Calibre,
    message::{Message, SharedStateMessage},
    types::{Precision, SoldierIndex, WorldPoint},
    utils::{BLUE, GREY},
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
    calibre: Calibre,
}

impl BulletFire {
    pub fn new(
        frame_i: u64,
        from: WorldPoint,
        to: WorldPoint,
        target: Option<(SoldierIndex, Precision)>,
        calibre: Calibre,
    ) -> Self {
        Self {
            new: true,
            start: frame_i,
            end: frame_i + 5,
            from,
            to,
            target,
            calibre,
        }
    }

    pub fn tick(&mut self, frame_i: u64) -> bool {
        println!(".");
        self.new = false;
        frame_i >= self.end
    }

    pub fn messages(&self, _frame_i: u64) -> Vec<Message> {
        // if self.new {
        //     return vec![Message::SharedState(SharedStateMessage::PushSoundToPlay(
        //         self.calibre.sound(),
        //     ))];
        // }

        vec![]
    }

    pub fn sprites(&self, _frame_i: u64) -> ggez::GameResult {
        Ok(())
    }

    pub fn meshes(
        &self,
        mesh_builder: &mut ggez::graphics::MeshBuilder,
        _frame_i: u64,
    ) -> ggez::GameResult {
        mesh_builder.line(&vec![self.from.to_vec2(), self.to.to_vec2()], 1.0, GREY)?;

        Ok(())
    }
}
