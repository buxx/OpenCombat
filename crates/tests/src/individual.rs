use bon::Builder;
use oc_individual::{
    BodyGesture, Gesture, Individual, Weapons,
    behavior::{Behavior, Intent},
};
use oc_mod::magazine::MagazineIndex;
use oc_root::{WorldConfig, geo::WorldVec3, side::Side};
use oc_utils::d2::Direction;

#[derive(Debug, Builder)]
pub struct TestIndividual {
    #[builder(default = Side::A)]
    side: Side,
    #[builder(default = WorldVec3::new(0., 0., 0.))]
    position: WorldVec3,
    #[builder(default = Behavior::Idle(Direction::NORTH))]
    behavior: Behavior,
    #[builder(default = Gesture::body(BodyGesture::StandUp(Direction::NORTH)))]
    gesture: Gesture,
    #[builder(default = Intent::Idle(Direction::NORTH))]
    intent: Intent,
    #[builder(default)]
    weapons: Weapons,
    #[builder(default)]
    magazines: Vec<MagazineIndex>,
}

impl TestIndividual {
    pub fn make(self, w: &WorldConfig) -> Individual {
        Individual::fresh(w, self.side, self.position)
            .with_gesture(self.gesture)
            .with_behavior(self.behavior)
            .with_intent(self.intent)
            .with_weapons(self.weapons)
            .with_magazines(self.magazines)
    }
}
