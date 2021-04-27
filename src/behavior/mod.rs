use crate::Vector2;

pub enum ItemBehavior {
    Standing(u32), // since
    Crawling,
    Walking(Vector2),
}
