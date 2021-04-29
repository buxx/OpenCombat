pub mod order;

use crate::ScenePoint;

pub enum ItemBehavior {
    Standing(u32), // since
    CrawlingTo(ScenePoint),
    WalkingTo(ScenePoint),
}
