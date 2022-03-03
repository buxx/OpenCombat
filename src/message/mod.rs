use crate::types::*;

#[derive(Debug)]
pub enum Message {
    Entity(EntityIndex, EntityMessage),
}

#[derive(Debug)]
pub enum EntityMessage {
    UpdateWorldPosition(WorldPosition),
}
