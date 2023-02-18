use ggez::{Context, GameResult};

use super::Engine;

impl Engine {
    pub fn tick(&mut self, ctx: &mut Context) -> GameResult {
        puffin::profile_scope!("tick");

        self.sync(ctx)?;

        // Collect player activity and react according to
        let mut messages = vec![];
        messages.extend(self.collect_player_inputs(ctx));
        messages.extend(self.ui_events(ctx));
        messages.extend(self.tick_interiors());
        messages.extend(self.tick_physics());
        self.react(messages, ctx)?;
        self.clean();

        Ok(())
    }

    pub fn clean(&mut self) {
        self.battle_state.clean(self.gui_state.get_frame_i());
    }
}
