use ggez::GameResult;

use crate::config::Config;

pub struct Network {
    config: Config,
}

impl Network {
    pub fn new(config: Config) -> GameResult<Self> {
        Ok(Self { config })
    }
}
