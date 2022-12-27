use ggez::Context;

use crate::message::*;

use super::Engine;

impl Engine {
    pub fn tick_as_client(&mut self, ctx: &mut Context) {
        // Client require a complete sync as first
        if self.local_state.is_first_frame() {
            self.network
                .send(vec![Message::Network(NetworkMessage::RequireCompleteSync)]);
        }

        // Grab and apply messages from server
        self.react(self.sync());
        self.react(self.deal_with_sync_errors_as_client());

        // Collect player activity and react according to
        let mut messages = vec![];
        messages.extend(self.collect_player_inputs(ctx));
        messages.extend(self.ui_events(ctx));
        messages.extend(self.tick_interiors());
        self.tick_physics(); // Client don't apply physics messages (only server)
        self.dispatch_as_client(&messages);
        self.react(messages);
    }
}
