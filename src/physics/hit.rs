use crate::gameplay::weapon::WeaponCharacteristic;
use crate::physics::util::meters_between_scene_points;
use crate::physics::visibility::Visibility;
use crate::physics::HitType;
use crate::scene::item::SceneItem;
use crate::{Factor, Meters};
use rand::Rng;

fn determine_miss_by_distance_factor(
    _visibility: &Visibility,
    from_scene_item: &SceneItem,
    distance: Meters,
) -> Factor {
    let mut factor = 1.0;

    for (distance_, factor_) in WeaponCharacteristic::new(&from_scene_item.weapon.type_)
        .miss_by_distance_factors
        .iter()
    {
        if distance >= *distance_ {
            break;
        }
        factor = *factor_
    }

    factor
}

fn determine_miss_by_opacity_factor(visibility: &Visibility) -> Factor {
    if visibility.path_final_opacity < 0.2 {
        1.0
    } else if visibility.path_final_opacity < 0.4 {
        2.0
    } else if visibility.path_final_opacity < 0.6 {
        3.0
    } else if visibility.path_final_opacity < 0.8 {
        4.0
    } else {
        5.0
    }
}

pub fn determine_hit_type(
    visibility: &Visibility,
    from_scene_item: &SceneItem,
    to_scene_item: &SceneItem,
) -> (i32, i32, HitType) {
    let miss_by_opacity_factor: Factor = determine_miss_by_opacity_factor(visibility);
    let miss_by_distance_factor: Factor = determine_miss_by_distance_factor(
        visibility,
        from_scene_item,
        meters_between_scene_points(&from_scene_item.position, &to_scene_item.position),
    );

    let mut rng = rand::thread_rng();
    let to_kill = 10;
    let to_incapacity = 30;
    let to_miss = 200.0;
    let max = to_kill
        + to_incapacity
        + (to_miss * miss_by_opacity_factor * miss_by_distance_factor) as i32;
    let hit_value = rng.gen_range(0..max);

    let mut hit_type = HitType::Missed;

    if hit_value <= to_kill {
        hit_type = HitType::Deadly;
    }

    if hit_value <= to_incapacity {
        hit_type = HitType::Incapacity;
    }

    (hit_value, max, hit_type)
}
