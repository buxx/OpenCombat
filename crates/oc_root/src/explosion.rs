use crate::physics::Meters;

#[derive(Debug, PartialEq)]
pub struct Explosion {
    pub kill_rayon: Meters,
    pub stunned_rayon: Meters,
    pub shrapnel_rayon: Meters,
    pub suppress_rayon: Meters,
    pub destruction_rayon: Meters,
    pub destruction_force: f32,
    pub shrapnel_quantity: u32,
    pub penetration_force: f32,
}

impl Explosion {
    pub fn largest_rayon(&self) -> f32 {
        self.kill_rayon
            .0
            .max(self.stunned_rayon.0)
            .max(self.shrapnel_rayon.0)
            .max(self.suppress_rayon.0)
            .max(self.destruction_rayon.0)
    }
}
