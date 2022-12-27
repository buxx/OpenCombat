use ggez::{graphics::MeshBuilder, GameResult};

use crate::message::Message;

use self::event::bullet::BulletFire;

pub mod event;
pub mod path;
pub mod utils;
pub mod visibility;

pub struct Physics {
    pub bullet_fires: Vec<BulletFire>,
}

impl Physics {
    pub fn new() -> Self {
        Self {
            bullet_fires: vec![],
        }
    }

    pub fn tick(&mut self, frame_i: u64) -> Vec<Message> {
        let mut messages = vec![];

        messages.extend(self.tick_bullet_fires(frame_i));

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

    pub fn draw(&self, mesh_builder: &mut MeshBuilder, frame_i: u64) -> GameResult {
        self.draw_bullet_fires(mesh_builder, frame_i)?;

        Ok(())
    }

    pub fn draw_bullet_fires(&self, mesh_builder: &mut MeshBuilder, frame_i: u64) -> GameResult {
        for bullet_fire in &self.bullet_fires {
            bullet_fire.sprites(frame_i)?;
            bullet_fire.meshes(mesh_builder, frame_i)?;
        }
        Ok(())
    }
}
