use std::collections::HashMap;

use ggez::Context;
use keyframe::{functions::Linear, keyframes, AnimationSequence};

use crate::{
    graphics::{soldier::SoldierAnimationType, AnimationFloor},
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

pub fn entity_animation(_entity: &ThreadSafeEntity) -> AnimationSequence<TweenableRect> {
    // FIXME BS NOW : hardcoded walk, must depend behavior

    let animation_type = SoldierAnimationType::Walking;

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

impl Graphics {
    pub fn initialize(&mut self, entities: &Vec<ThreadSafeEntity>) {
        self.entity_animation_sequences = HashMap::new();

        for (i, entity) in entities.iter().enumerate() {
            let animation = entity_animation(entity);
            self.entity_animation_sequences
                .insert(EntityIndex(i), animation);
        }
    }

    pub fn update(&mut self, ctx: &Context) {
        let secs = ggez::timer::delta(ctx).as_secs_f64();
        for (_, animation) in self.entity_animation_sequences.iter_mut() {
            animation.advance_and_maybe_wrap(secs);
        }
    }
}
