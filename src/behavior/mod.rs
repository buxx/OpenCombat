pub mod animate;
pub mod order;

use crate::ScenePoint;

pub enum ItemBehavior {
    Standing, // since
    CrawlingTo(ScenePoint),
    WalkingTo(ScenePoint),
}
