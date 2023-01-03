use crate::{
    audio::Sound,
    engine::Engine,
    message::{Message, SharedStateMessage},
    physics::effect::Effect,
};
use rand::seq::SliceRandom;

impl Engine {
    pub fn fx_effects(&self) -> Vec<Message> {
        let mut messages = vec![];

        for effect in self.shared_state.physics_effects() {
            match effect {
                Effect::KillingBlast(soldier_index) => {
                    let soldier = self.shared_state.soldier(*soldier_index);
                    if soldier.can_produce_sound() {
                        let pick_from = vec![
                            Sound::MaleScreaming1,
                            Sound::MaleScreaming2,
                            Sound::MaleScreaming3,
                            Sound::MaleScreaming4,
                            Sound::MaleScreaming5,
                            Sound::MaleDie1,
                            Sound::MaleDie2,
                            Sound::MaleDie3,
                            Sound::MaleDie4,
                            Sound::MaleDie5,
                            Sound::MaleDie6,
                            Sound::MaleDie7,
                            Sound::MaleDie8,
                        ];
                        messages.push(Message::SharedState(SharedStateMessage::PushSoundToPlay(
                            *pick_from
                                .choose(&mut rand::thread_rng())
                                .expect("Must one be chosen"),
                        )))
                    }
                }
                Effect::StunningBlast(_soldier_index) => {
                    // TODO
                }
                Effect::ProximityBlast(_, _) => {}
            }
        }

        messages
    }

    pub fn resolve_effects(&mut self) -> Vec<Message> {
        let mut messages = vec![];

        while let Some(effect) = self.shared_state.physics_effects_mut().pop() {
            match effect {
                Effect::KillingBlast(soldier_index) => {
                    messages.extend(self.soldier_die(soldier_index))
                }
                Effect::StunningBlast(soldier_index) => {
                    messages.extend(self.soldier_stunned(soldier_index))
                }
                Effect::ProximityBlast(soldier_index, distance) => {
                    messages.extend(self.soldier_blast(soldier_index, distance))
                }
            }
        }

        messages
    }
}
