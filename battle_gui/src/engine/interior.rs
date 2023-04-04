use battle_core::types::WorldPoint;
use ggez::graphics::{DrawParam, Rect};

use crate::{engine::message::EngineMessage, graphics::batch::QualifiedBatch};

use super::Engine;

impl Engine {
    pub fn tick_interiors(&mut self) -> Vec<EngineMessage> {
        puffin::profile_scope!("tick_interiors");
        let messages = vec![];
        let tick_update = self.gui_state.get_frame_i() % self.config.interiors_update_freq == 0;

        if tick_update {
            self.update_interior_sprites();
        }

        messages
    }

    pub fn update_interior_sprites(&mut self) {
        self.graphics
            .clear_map_interiors_batch(&self.gui_state.zoom);

        for interior in self.battle_state.map().interiors() {
            // World coordinates
            let start_x = interior.x();
            let start_y = interior.y();
            let end_x = start_x + interior.width();
            let end_y = start_y + interior.height();

            for soldier in self.battle_state.soldiers() {
                let can_see_interior = soldier.can_see_interior();
                let is_current_side = soldier.get_side() == self.gui_state.side();

                if !can_see_interior || !is_current_side {
                    continue;
                }

                let soldier_position = soldier.get_world_point();
                if soldier_position.x >= start_x
                    && soldier_position.x <= end_x
                    && soldier_position.y >= start_y
                    && soldier_position.y <= end_y
                {
                    self.graphics.interiors_mut().push(
                        &self.gui_state.zoom,
                        DrawParam::new()
                            .src(Rect::new(
                                interior.relative_x(),
                                interior.relative_y(),
                                interior.relative_width(),
                                interior.relative_height(),
                            ))
                            .dest(WorldPoint::new(start_x, start_y).to_vec2()),
                    );
                    continue;
                }
            }
        }
    }
}
