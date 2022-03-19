use ggez::Context;

use super::Engine;

impl Engine {
    pub fn tick_as_server(&mut self, ctx: &mut Context) {
        // Will collect all tick messages
        let mut messages = vec![];

        // Compute entities
        messages.extend(self.tick_entities());

        // Retrieve messages from clients
        messages.extend(self.sync());

        // Check any network errors
        messages.extend(self.deal_with_sync_errors_as_server());

        // Retrieve messages from user inputs
        messages.extend(self.collect_player_inputs(ctx));

        // Generate messages according to the possible ui events
        messages.extend(self.ui_events(ctx));

        // Apply messages
        self.react(messages);
    }
}
