use bevy::prelude::*;

use crate::{entity::individual::IndividualIndex, ingame::individual::Status, world::World};

const OPERATIONAL: Color = Color::srgba(0.1, 0.9, 0.1, 0.2);
const STUNNED: Color = Color::srgba(1.0, 0.55, 0.0, 0.3);
const DEAD: Color = Color::srgba(0.9, 0.1, 0.1, 0.2);
const LEADER_ALPHA: f32 = 0.5;

#[derive(Debug, Component)]
pub struct IndividualOutline;

// Warning: AI generated
/// Keep outline on the same animation frame than its individual, and colored according to status
pub fn sync_individual_outlines(
    world: Res<World>,
    individuals: Query<(&IndividualIndex, &Sprite, &Status), Without<IndividualOutline>>,
    mut outlines: Query<(&ChildOf, &mut Sprite), With<IndividualOutline>>,
) {
    for (parent, mut outline) in &mut outlines {
        let Ok((index, sprite, status)) = individuals.get(parent.parent()) else {
            continue;
        };

        let mut color = match status.0 {
            oc_individual::Status::Operational => OPERATIONAL,
            oc_individual::Status::Stunned(_) => STUNNED,
            oc_individual::Status::Dead => DEAD,
        };
        let is_leader = world
            .individual_squad(index.0)
            .and_then(|(squad, _)| world.squad(squad))
            .is_some_and(|squad| squad.leader() == index.0);
        if is_leader {
            color.set_alpha(LEADER_ALPHA);
        }
        if outline.color != color {
            outline.color = color;
        }

        let frame = sprite.texture_atlas.as_ref().map(|atlas| atlas.index);
        if let (Some(frame), Some(atlas)) = (frame, outline.texture_atlas.as_mut())
            && atlas.index != frame
        {
            atlas.index = frame;
        }
    }
}
