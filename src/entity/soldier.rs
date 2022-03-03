pub struct Soldier {
    pub world_position: WorldPosition,
}

impl Entity for Soldier {
    fn world_position(&self) -> &WorldPosition {
        &self.world_position
    }
}
