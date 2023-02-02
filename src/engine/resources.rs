use ggez::{Context, GameResult};

use super::Engine;

impl Engine {
    pub fn update_resources(&mut self, ctx: &mut Context) -> GameResult<()> {
        let messages = vec![];

        let mut buffer = [0u8; 4096];
        if self.map_watcher.read_events(&mut buffer).is_ok() {
            self.graphics.reload_map(ctx, &self.map)?;
        };

        let mut buffer = [0u8; 4096];
        if self.resources_watcher.read_events(&mut buffer).is_ok() {
            self.graphics.update_resources()?;
        };

        let side_effects = self.react(messages, ctx)?;
        self.react_side_effects(side_effects, ctx);
        Ok(())
    }
}
