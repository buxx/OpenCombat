use battle_core::{
    entity::soldier::{Soldier, WeaponClass},
    game::{explosive::ExplosiveType, weapon::WeaponSprite, Side},
    graphics::{cannon_blast::CannonBlastAnimationType, soldier::SoldierAnimationType, Sprite},
    types::{Angle, WorldPoint},
};
use ggez::Context;
use keyframe::{
    functions::{EaseOut, Linear},
    keyframes, AnimationSequence,
};

use super::{soldier::SoldierAnimationSequence, AnimationFloor, Graphics, TweenableRect};

impl Graphics {
    pub fn refresh_soldier_animation(&mut self, soldier: &Soldier) {
        let animation = self.soldier_animation(soldier);
        self.soldier_animation_sequences
            .insert(soldier.uuid(), animation);
    }

    pub fn push_explosion_animation(&mut self, point: WorldPoint, type_: ExplosiveType) {
        let animation = self.explosion_animation(type_);
        self.explosion_sequences.push((point, animation));
    }

    pub fn push_canon_blast_animation(
        &mut self,
        point: WorldPoint,
        angle: Angle,
        type_: WeaponSprite,
        soldier_animation_type: SoldierAnimationType,
    ) {
        let animation = self.canon_blast_animation(type_, soldier_animation_type);
        self.canon_blast_sequences.push((point, angle, animation));
    }

    pub fn remove_explosion_animation(&mut self, point: WorldPoint) {
        self.explosion_sequences.retain(|e| e.0 != point)
    }

    pub fn remove_canon_blast_animation(&mut self, point: WorldPoint) {
        self.canon_blast_sequences.retain(|e| e.0 != point);
    }

    pub fn update(&mut self, ctx: &Context) {
        let secs = ctx.time.delta().as_secs_f64();

        for (_, animation) in self.soldier_animation_sequences.iter_mut() {
            animation.soldier_mut().advance_and_maybe_wrap(secs);
            if let Some(weapon_animation) = animation.weapon_mut() {
                weapon_animation.advance_and_maybe_wrap(secs);
            }
        }
        for (_, animation) in self.explosion_sequences.iter_mut() {
            animation.advance_and_maybe_wrap(secs);
        }
    }

    pub fn soldier_animation(&self, soldier: &Soldier) -> SoldierAnimationSequence {
        let (soldier_animation_type, weapon_animation_type) = soldier.animation_type();

        let soldier_src_rect_start = TweenableRect::new(
            soldier_animation_type.src_x_start(),
            soldier_animation_type.src_y(soldier.side()),
            soldier_animation_type.width(),
            soldier_animation_type.height(),
        );
        let soldier_src_end_rect = TweenableRect::new(
            soldier_animation_type.src_x_end(),
            soldier_animation_type.src_y(soldier.side()),
            soldier_animation_type.width(),
            soldier_animation_type.height(),
        );
        let duration = soldier_animation_type.duration();

        let soldier_easing = AnimationFloor {
            pre_easing: Box::new(Linear),
            frames: soldier_animation_type.frame_count() as i32,
        };

        let soldier_animation_sequence = keyframes![
            (soldier_src_rect_start, 0.0, soldier_easing),
            (soldier_src_end_rect, duration)
        ];

        let weapon_animation_sequence = if let Some(_weapon) = soldier.weapon(&WeaponClass::Main) {
            // FIXME : weapon Y (current Side in src_y function ...)
            let weapon_src_rect_start = TweenableRect::new(
                weapon_animation_type.src_x_start(),
                weapon_animation_type.src_y(&Side::A), // Fake Side because weapon sprite uniques
                weapon_animation_type.width(),
                weapon_animation_type.height(),
            );
            let weapon_src_end_rect = TweenableRect::new(
                weapon_animation_type.src_x_end(),
                weapon_animation_type.src_y(&Side::A), // Fake Side because weapon sprite uniques
                weapon_animation_type.width(),
                weapon_animation_type.height(),
            );

            let weapon_easing = AnimationFloor {
                pre_easing: Box::new(Linear),
                frames: weapon_animation_type.frame_count() as i32,
            };

            Some(keyframes![
                (weapon_src_rect_start, 0.0, weapon_easing),
                (weapon_src_end_rect, duration)
            ])
        } else {
            None
        };

        SoldierAnimationSequence::new(soldier_animation_sequence, weapon_animation_sequence)
    }

    pub fn explosion_animation(&self, type_: ExplosiveType) -> AnimationSequence<TweenableRect> {
        let sprite = type_.sprite();

        let src_rect_start = TweenableRect::new(
            sprite.src_x_start(),
            sprite.src_y(&Side::All),
            sprite.width(),
            sprite.height(),
        );
        let src_end_rect = TweenableRect::new(
            sprite.src_x_end(),
            sprite.src_y(&Side::All),
            sprite.width(),
            sprite.height(),
        );
        let duration = sprite.duration();

        let easing = AnimationFloor {
            pre_easing: Box::new(EaseOut),
            frames: sprite.frame_count() as i32,
        };

        keyframes![(src_rect_start, 0.0, easing), (src_end_rect, duration)]
    }

    pub fn canon_blast_animation(
        &self,
        type_: WeaponSprite,
        soldier_animation_type: SoldierAnimationType,
    ) -> AnimationSequence<TweenableRect> {
        let animation_type = CannonBlastAnimationType::from((type_, soldier_animation_type));

        let src_rect_start = TweenableRect::new(
            animation_type.src_x_start(),
            animation_type.src_y(&Side::A), // TODO: Give side here is not correct
            animation_type.width(),
            animation_type.height(),
        );
        let src_end_rect = TweenableRect::new(
            animation_type.src_x_end(),
            animation_type.src_y(&Side::A), // TODO: Give side here is not correct
            animation_type.width(),
            animation_type.height(),
        );
        let duration = animation_type.duration();

        let easing = AnimationFloor {
            pre_easing: Box::new(Linear),
            frames: animation_type.frame_count() as i32,
        };

        keyframes![(src_rect_start, 0.0, easing), (src_end_rect, duration)]
    }
}
