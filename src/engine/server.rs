use ggez::Context;

use super::Engine;

impl Engine {
    pub fn tick_as_server(&mut self, ctx: &mut Context) {
        // Grab and apply messages from clients
        self.react(self.sync());
        self.react(self.deal_with_sync_errors_as_server());

        let mut messages = vec![];
        messages.extend(self.tick_soldiers());
        messages.extend(self.tick_feeling_decreasing_soldiers());
        messages.extend(self.tick_interiors());
        messages.extend(self.tick_visibilities());
        messages.extend(self.tick_physics());
        messages.extend(self.collect_player_inputs(ctx));
        messages.extend(self.ui_events(ctx));
        self.dispatch_as_server(&messages);

        let side_effects = self.react(messages);
        self.react_side_effects(side_effects, ctx);
        self.local_state.remove_finished_physics();
    }
}
