use super::{Runner, RunnerError};

impl Runner {
    pub fn tick(&mut self) -> Result<(), RunnerError> {
        let frame_i = self.frame_i;
        puffin::profile_scope!("tick", format!("frame {frame_i}"));
        self.inputs()?;

        let mut messages = vec![];
        messages.extend(self.tick_phase());
        messages.extend(self.tick_soldiers());
        messages.extend(self.tick_feeling_decreasing_soldiers());
        messages.extend(self.tick_visibilities());
        messages.extend(self.tick_physics());
        self.react(&messages);
        self.clean();

        self.outputs(&messages)?;
        Ok(())
    }

    pub fn clean(&mut self) {
        self.battle_state.clean(self.frame_i);
    }
}
