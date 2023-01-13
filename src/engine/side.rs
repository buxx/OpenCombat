use ggez::Context;

use crate::{behavior::Behavior, order::Order, state::SideEffect};

use super::Engine;

impl Engine {
    pub fn react_side_effects(&mut self, side_effects: Vec<SideEffect>, ctx: &mut Context) {
        for side_effect in side_effects {
            match side_effect {
                SideEffect::RefreshEntityAnimation(soldier_index) => {
                    let soldier = self.shared_state.soldier(soldier_index);
                    self.graphics
                        .refresh_soldier_animation(soldier_index, soldier);
                }
                SideEffect::SoldierFinishHisBehavior(soldier_index) => {
                    let soldier = self.shared_state.soldier_mut(soldier_index);
                    soldier.set_behavior(Behavior::Idle);
                    soldier.set_order(Order::Idle);
                }
                SideEffect::PlaySound(sound) => {
                    if let Some(player) = &mut self.player {
                        match player.play(sound, ctx) {
                            Err(e) => println!("ERROR :: Sound error :: {e}"),
                            _ => {}
                        };
                    }
                }
            }
        }
    }
}
