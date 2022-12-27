use std::collections::HashMap;

use ggez::Context;
use keyframe::{
    functions::{EaseOut, Linear},
    keyframes, AnimationSequence,
};

use crate::{
    entity::soldier::Soldier, game::explosive::Type as ExplosiveType, graphics::AnimationFloor,
    types::*,
};

use super::{Graphics, TweenableRect};

pub trait Sprite {
    fn sprite_sheet_column_count(&self) -> usize;
    fn sprite_sheet_row_count(&self) -> usize;
    fn src_x_start(&self) -> f32;
    fn src_x_end(&self) -> f32;
    fn src_y(&self) -> f32;
    fn frame_count(&self) -> usize;
    fn width(&self) -> f32;
    fn height(&self) -> f32;
    fn duration(&self) -> f32;
}

pub fn soldier_animation(soldier: &Soldier) -> AnimationSequence<TweenableRect> {
    let animation_type = soldier.get_animation_type();

    let src_rect_start = TweenableRect::new(
        animation_type.src_x_start(),
        animation_type.src_y(),
        animation_type.width(),
        animation_type.height(),
    );
    let src_end_rect = TweenableRect::new(
        animation_type.src_x_end(),
        animation_type.src_y(),
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

pub fn explosion_animation(type_: ExplosiveType) -> AnimationSequence<TweenableRect> {
    let sprite = type_.sprite();

    let src_rect_start = TweenableRect::new(
        sprite.src_x_start(),
        sprite.src_y(),
        sprite.width(),
        sprite.height(),
    );
    let src_end_rect = TweenableRect::new(
        sprite.src_x_end(),
        sprite.src_y(),
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

impl Graphics {
    pub fn initialize(&mut self, soldiers: &Vec<Soldier>) {
        self.soldier_animation_sequences = HashMap::new();

        for (i, soldier) in soldiers.iter().enumerate() {
            self.refresh_soldier_animation(SoldierIndex(i), soldier)
        }
    }

    pub fn refresh_soldier_animation(&mut self, soldier_index: SoldierIndex, soldier: &Soldier) {
        let animation = soldier_animation(soldier);
        self.soldier_animation_sequences
            .insert(soldier_index, animation);
    }

    pub fn push_explosion_animation(&mut self, point: WorldPoint, type_: ExplosiveType) {
        let animation = explosion_animation(type_);
        self.explosion_sequences.push((point, animation));
    }

    pub fn remove_explosion_animation(&mut self, point: WorldPoint) {
        self.explosion_sequences.retain(|e| e.0 != point)
    }

    pub fn update(&mut self, ctx: &Context) {
        let secs = ggez::timer::delta(ctx).as_secs_f64();

        for (_, animation) in self.soldier_animation_sequences.iter_mut() {
            animation.advance_and_maybe_wrap(secs);
        }
        for (_, animation) in self.explosion_sequences.iter_mut() {
            animation.advance_and_maybe_wrap(secs);
        }
    }
}
