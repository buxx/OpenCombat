use battle_core::{
    audio::Sound,
    entity::soldier::Soldier,
    physics::{
        coverage::SoldierCovered, event::bullet::BulletFire, utils::distance_between_points,
    },
    state::client::ClientStateMessage,
    types::Distance,
};
use rand::seq::SliceRandom;
use rand::Rng;

use crate::runner::{message::RunnerMessage, Runner};

impl Runner {
    pub fn tick_bullet_fires(&self) -> Vec<RunnerMessage> {
        puffin::profile_scope!("tick_bullet_fires");
        let mut messages = vec![];

        for bullet_fire in self.battle_state.bullet_fires() {
            if bullet_fire.effective(*self.battle_state.frame_i()) {
                messages.extend(self.bullet_fire_effects(bullet_fire))
            }
        }

        messages
    }

    // FIXME : find algorithm kill/injure about bullet + terrain + position
    fn bullet_fire_effects(&self, bullet_fire: &BulletFire) -> Vec<RunnerMessage> {
        puffin::profile_scope!(
            "bullet_fire_effects",
            format!("start={} end={}", bullet_fire.start(), bullet_fire.end())
        );
        let mut messages = vec![];
        let point = bullet_fire.point();

        for soldier in self.battle_state.soldiers() {
            if !soldier.can_feel_bullet_fire() {
                continue;
            }

            // Simple for now, but if in vehicle, don't be affected
            if self
                .battle_state
                .soldier_vehicle_place(soldier.uuid())
                .is_some()
            {
                continue;
            }

            let from = &soldier.world_point();
            let distance = distance_between_points(from, point);

            if distance.meters() < 1
                && SoldierCovered::new(self.battle_state.map(), bullet_fire, soldier).compute()
            {
                messages.extend(self.covered_bullet_effects(soldier));
                messages.extend(self.proximity_bullet_effects(soldier, &distance))
            } else if distance.millimeters() < 500 {
                let mut rng = rand::thread_rng();
                let value: u8 = rng.gen();
                if value < 10 {
                    messages.extend(self.killing_bullet_effects(soldier))
                } else if value < 50 {
                    messages.extend(self.injuring_bullet_effects(soldier))
                } else {
                    messages.extend(self.proximity_bullet_effects(soldier, &distance))
                }
            } else if distance.meters() < 30 {
                messages.extend(self.proximity_bullet_effects(soldier, &distance))
            }
        }

        messages
    }

    pub fn killing_bullet_effects(&self, soldier: &Soldier) -> Vec<RunnerMessage> {
        puffin::profile_scope!("KillingBullet", soldier.uuid().to_string());
        let mut messages = self.soldier_die(soldier.uuid());

        let soldier = self.battle_state.soldier(soldier.uuid());
        if soldier.can_produce_sound() {
            let pick_from = [
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
            messages.push(RunnerMessage::ClientsState(
                ClientStateMessage::PlayBattleSound(
                    *pick_from
                        .choose(&mut rand::thread_rng())
                        .expect("Must one be chosen"),
                ),
            ))

            // TODO (bullet flesh sound impact)
        }

        messages
    }

    pub fn injuring_bullet_effects(&self, soldier: &Soldier) -> Vec<RunnerMessage> {
        puffin::profile_scope!("InjuringBullet", soldier.uuid().to_string());
        let mut messages = self.soldier_bullet_injured(soldier.uuid());

        let soldier = self.battle_state.soldier(soldier.uuid());
        if soldier.can_produce_sound() {
            let pick_from = [
                Sound::MaleDie1,
                Sound::MaleDie2,
                Sound::MaleDie3,
                Sound::MaleDie4,
                Sound::MaleDie5,
                Sound::MaleDie6,
                Sound::MaleDie7,
                Sound::MaleDie8,
            ];
            messages.push(RunnerMessage::ClientsState(
                ClientStateMessage::PlayBattleSound(
                    *pick_from
                        .choose(&mut rand::thread_rng())
                        .expect("Must one be chosen"),
                ),
            ))
        }

        messages
    }

    pub fn covered_bullet_effects(&self, soldier: &Soldier) -> Vec<RunnerMessage> {
        puffin::profile_scope!("covered_bullet_effects", soldier.uuid().to_string());
        let pick_from = [
            Sound::BulletMetalImpact1,
            Sound::BulletTrunkImpact1,
            Sound::BulletWallImpact1,
            Sound::BulletGroundImpact1,
        ];
        vec![RunnerMessage::ClientsState(
            ClientStateMessage::PlayBattleSound(
                *pick_from
                    .choose(&mut rand::thread_rng())
                    .expect("Must one be chosen"),
            ),
        )]
    }

    pub fn proximity_bullet_effects(
        &self,
        soldier: &Soldier,
        distance: &Distance,
    ) -> Vec<RunnerMessage> {
        puffin::profile_scope!("proximity_bullet_effects", soldier.uuid().to_string());
        self.soldier_proximity_bullet(soldier.uuid(), distance)
    }
}
