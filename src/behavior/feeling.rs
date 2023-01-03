use crate::types::FeelingIntensity;

pub enum Feeling {
    UnderFire,
    Fear,
}

impl Feeling {
    pub fn decrease_value(&self) -> FeelingIntensity {
        match self {
            Feeling::UnderFire => FeelingIntensity(10),
            Feeling::Fear => FeelingIntensity(10),
        }
    }

    pub fn max(&self) -> FeelingIntensity {
        match self {
            Feeling::UnderFire => FeelingIntensity(200),
            Feeling::Fear => FeelingIntensity(100),
        }
    }
}
