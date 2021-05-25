use crate::physics::visibility::Visibility;
use crate::physics::HitType;
use crate::scene::item::SceneItem;
use rand::Rng;

pub fn determine_hit_type(
    visibility: &Visibility,
    _from_scene_item: &SceneItem,
    _to_scene_item: &SceneItem,
) -> HitType {
    let miss_by_opacity_factor: i32 = if visibility.path_final_opacity < 0.2 {
        1
    } else if visibility.path_final_opacity < 0.4 {
        2
    } else if visibility.path_final_opacity < 0.6 {
        3
    } else if visibility.path_final_opacity < 0.8 {
        4
    } else {
        5
    };

    let mut rng = rand::thread_rng();
    let to_kill = 10;
    let to_incapacity = 30;
    let to_miss = 200;
    let max = to_kill + to_incapacity + (to_miss * miss_by_opacity_factor);
    let hit_value = rng.gen_range(0..max);

    if hit_value <= to_kill {
        return HitType::Deadly;
    }

    if hit_value <= to_incapacity {
        return HitType::Incapacity;
    }

    HitType::Missed
}
