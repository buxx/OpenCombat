use ggez::{Context, GameResult};

use super::{message::EngineMessage, Engine};

impl Engine {
    pub fn tick(&mut self, ctx: &mut Context) -> GameResult {
        puffin::profile_scope!("tick");

        self.sync(ctx)?;

        // Collect player activity and react according to
        let mut messages = vec![];
        messages.extend(self.tick_state());
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

    pub fn tick_state(&self) -> Vec<EngineMessage> {
        let mut messages = vec![];

        messages.extend(self.tick_intro());

        messages
    }
}
