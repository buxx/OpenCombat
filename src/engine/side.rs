use crate::state::SideEffect;

use super::Engine;

impl Engine {
    pub fn react_side_effects(&mut self, side_effects: Vec<SideEffect>) {
        for side_effect in side_effects {
            match side_effect {
                SideEffect::RefreshEntityAnimation(soldier_index) => {
                    let soldier = self.shared_state.soldier(soldier_index);
                    self.graphics
                        .refresh_soldier_animation(soldier_index, soldier);
                }
            }
        }
    }
}
