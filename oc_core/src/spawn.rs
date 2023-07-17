use std::{fmt::Display, str::FromStr};

#[derive(Clone, Debug, PartialEq)]
pub enum SpawnZoneName {
    North,
    NorthEst,
    Est,
    SouthEst,
    South,
    SouthWest,
    West,
    NorthWest,
    All,
}
impl SpawnZoneName {
    pub fn allowed_for_zone_object(&self) -> bool {
        !matches!(self, SpawnZoneName::All)
    }
}

#[derive(Clone, Debug)]
pub struct ParseOriginDirectionError(String);

impl Display for ParseOriginDirectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for SpawnZoneName {
    type Err = ParseOriginDirectionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "N" => Ok(Self::North),
            "NE" => Ok(Self::NorthEst),
            "E" => Ok(Self::Est),
            "SE" => Ok(Self::SouthEst),
            "S" => Ok(Self::South),
            "SW" => Ok(Self::SouthWest),
            "W" => Ok(Self::West),
            "NW" => Ok(Self::NorthWest),
            "ALL" => Ok(Self::All),
            _ => Err(ParseOriginDirectionError(format!(
                "Unknown spawn zone name '{}'",
                s
            ))),
        }
    }
}
