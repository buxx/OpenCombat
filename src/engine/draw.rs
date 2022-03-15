use ggez::{graphics::MeshBuilder, GameResult};

use super::Engine;

impl Engine {
    // TODO : don't generate sprites of non visible entities (hidden enemy, outside screen, etc)
    pub fn generate_entities_sprites(&mut self) -> GameResult {
        for entity in self.shared_state.entities() {
            for sprite in self.graphics.entity_sprites(entity) {
                let sprite_ = sprite.dest(entity.get_world_point().to_vec2());
                self.graphics.append_sprites_batch(sprite_);
            }
        }

        Ok(())
    }

    pub fn generate_map_sprites(&self, _draw_decor: bool) -> GameResult {
        // Note : Background sprites have been prepared once for map_background_batch
        // Note : Decor sprites have been prepared once for map_background_batch
        Ok(())
    }

    pub fn generate_menu_sprites(&mut self) -> GameResult {
        if let Some((to_point, squad_id)) = self.local_state.get_squad_menu() {
            for sprite in self.graphics.squad_menu_sprites(
                *to_point,
                *self.local_state.get_current_cursor_window_point(),
                *squad_id,
            ) {
                self.graphics.append_ui_batch(sprite);
            }
        }

        Ok(())
    }

    pub fn generate_selection_meshes(&self, mesh_builder: &mut MeshBuilder) -> GameResult {
        self.generate_selected_entities_meshes(mesh_builder)?;

        Ok(())
    }

    pub fn generate_debug_meshes(&self, mesh_builder: &mut MeshBuilder) -> GameResult {
        if self.local_state.get_debug().mouse() {
            self.generate_debug_mouse_meshes(mesh_builder)?;
        }

        self.generate_select_rectangle_meshes(mesh_builder)?;

        Ok(())
    }
}
