// WARNING: AI generated
//! Picking backend which consider entity as hit when cursor is near its center. It extends the
//! default sprite picking (which only hit on opaque pixels) for small sprites hard to click.
use bevy::picking::backend::prelude::*;
use bevy::prelude::*;

/// Entities owning this component are hit when the cursor is within given (world pixels) radius
/// around their center.
#[derive(Debug, Component, Clone, Copy)]
pub struct PickRadius(pub f32);

pub struct PickRadiusPlugin;

impl Plugin for PickRadiusPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, pick.in_set(PickingSystems::Backend));
    }
}

fn pick(
    ray_map: Res<RayMap>,
    cameras: Query<(&Camera, &GlobalTransform, &Projection)>,
    targets: Query<(Entity, &PickRadius, &GlobalTransform, &ViewVisibility), With<Pickable>>,
    mut writer: MessageWriter<PointerHits>,
) {
    for (ray_id, ray) in ray_map.iter() {
        let Ok((camera, camera_transform, Projection::Orthographic(ortho))) =
            cameras.get(ray_id.camera)
        else {
            continue;
        };
        if !camera.is_active {
            continue;
        }

        let cursor = ray.origin.truncate();
        let camera_inverse = camera_transform.affine().inverse();
        let picks = targets
            .iter()
            .filter(|(_, _, _, visibility)| visibility.get())
            .filter_map(|(entity, radius, transform, _)| {
                let position = transform.translation();
                (position.truncate().distance(cursor) <= radius.0).then(|| {
                    // Depth from camera near plane (same computation than sprite backend)
                    let depth = -ortho.near - camera_inverse.transform_point3(position).z;
                    let hit = HitData::new(
                        ray_id.camera,
                        depth,
                        Some(position),
                        Some(*transform.back()),
                    );
                    (entity, hit)
                })
            })
            .collect::<Vec<_>>();

        if !picks.is_empty() {
            writer.write(PointerHits::new(ray_id.pointer, picks, camera.order as f32));
        }
    }
}
