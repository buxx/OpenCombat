use battle_core::{
    entity::soldier::Soldier,
    game::{explosive::ExplosiveType, Side},
    types::WorldPoint,
};
use ggez::Context;
use keyframe::{
    functions::{EaseOut, Linear},
    keyframes, AnimationSequence,
};

use super::{AnimationFloor, Graphics, TweenableRect};

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

    pub fn remove_explosion_animation(&mut self, point: WorldPoint) {
        self.explosion_sequences.retain(|e| e.0 != point)
    }

    pub fn update(&mut self, ctx: &Context) {
        let secs = ctx.time.delta().as_secs_f64();

        for (_, animation) in self.soldier_animation_sequences.iter_mut() {
            animation.advance_and_maybe_wrap(secs);
        }
        for (_, animation) in self.explosion_sequences.iter_mut() {
            animation.advance_and_maybe_wrap(secs);
        }
    }

    pub fn soldier_animation(&self, soldier: &Soldier) -> AnimationSequence<TweenableRect> {
        let animation_type = self.soldier_animation_type(soldier);

        let src_rect_start = TweenableRect::new(
            animation_type.src_x_start(),
            animation_type.src_y(soldier.side()),
            animation_type.width(),
            animation_type.height(),
        );
        let src_end_rect = TweenableRect::new(
            animation_type.src_x_end(),
            animation_type.src_y(soldier.side()),
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
}
