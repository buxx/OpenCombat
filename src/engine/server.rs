use super::Engine;

impl Engine {
    pub fn tick_as_server(&mut self) {
        // Will collect all tick messages
        let mut messages = vec![];

        // Compute entities
        messages.extend(self.tick_entities());

        // Retrieve messages from clients
        messages.extend(self.sync());

        // Check any network errors
        messages.extend(self.deal_with_errors_as_server());

        // Apply messages
        self.react(messages);
    }
}
