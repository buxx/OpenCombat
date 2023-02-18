use ggez::{graphics::MeshBuilder, GameResult};

use crate::{debug::DebugPhysics, engine::message::EngineMessage};

use super::{input::Control, Engine};

mod bullet;
mod explosion;

impl Engine {
    // FIXME BS NOW : as GUI only !
    pub fn tick_physics(&mut self) -> Vec<EngineMessage> {
        puffin::profile_scope!("tick_physics");
        let mut messages = vec![];

        messages.extend(self.tick_bullet_fires());
        messages.extend(self.tick_explosions());

        messages
    }

    pub fn draw_physics(&self, mesh_builder: &mut MeshBuilder) -> GameResult {
        self.draw_bullet_fires(mesh_builder)?;
        self.draw_explosions(mesh_builder)?;

        Ok(())
    }

    pub fn physics_control(&self, physics: &DebugPhysics) -> Control {
        match physics {
            DebugPhysics::None => Control::Soldiers,
            _ => Control::Physics,
        }
    }
}
