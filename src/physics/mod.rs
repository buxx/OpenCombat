use ggez::{graphics::MeshBuilder, GameResult};

use crate::{message::Message, state::local::LocalState, utils::GREY};

use self::event::{bullet::BulletFire, explosion::Explosion};

pub mod event;
pub mod path;
pub mod utils;
pub mod visibility;

pub struct Physics {
    pub bullet_fires: Vec<BulletFire>,
    pub explosions: Vec<Explosion>,
}

impl Physics {
    pub fn new() -> Self {
        Self {
            bullet_fires: vec![],
            explosions: vec![],
        }
    }

    pub fn tick(&mut self, frame_i: u64) -> Vec<Message> {
        let mut messages = vec![];

        messages.extend(self.tick_bullet_fires(frame_i));
        messages.extend(self.tick_explosions(frame_i));

        messages
    }

    pub fn tick_bullet_fires(&mut self, frame_i: u64) -> Vec<Message> {
        let mut messages = vec![];

        let mut to_removes = vec![];
        for (i, bullet_fire) in self.bullet_fires.iter_mut().enumerate() {
            messages.extend(bullet_fire.messages(frame_i));
            if bullet_fire.tick(frame_i) {
                to_removes.push(i)
            }
        }
        to_removes.reverse();
        for i in to_removes {
            self.bullet_fires.remove(i);
        }

        messages
    }

    pub fn tick_explosions(&mut self, frame_i: u64) -> Vec<Message> {
        let mut messages = vec![];

        let mut to_removes = vec![];
        for (i, explosion) in self.explosions.iter_mut().enumerate() {
            messages.extend(explosion.messages(frame_i));
            if explosion.tick(frame_i) {
                to_removes.push(i)
            }
        }
        to_removes.reverse();
        for i in to_removes {
            self.explosions.remove(i);
        }

        messages
    }

    pub fn draw(&self, mesh_builder: &mut MeshBuilder, local_state: &LocalState) -> GameResult {
        self.draw_bullet_fires(mesh_builder, local_state)?;
        self.draw_explosions(mesh_builder, local_state)?;

        Ok(())
    }

    pub fn draw_bullet_fires(
        &self,
        mesh_builder: &mut MeshBuilder,
        local_state: &LocalState,
    ) -> GameResult {
        for bullet_fire in &self.bullet_fires {
            let from = local_state.window_point_from_world_point(*bullet_fire.from());
            let to = local_state.window_point_from_world_point(*bullet_fire.to());
            mesh_builder.line(&vec![from.to_vec2(), to.to_vec2()], 1.0, GREY)?;
        }
        Ok(())
    }

    pub fn draw_explosions(
        &self,
        _mesh_builder: &mut MeshBuilder,
        _local_state: &LocalState,
    ) -> GameResult {
        // Nothing here because drawn by graphics sequences
        Ok(())
    }
}
