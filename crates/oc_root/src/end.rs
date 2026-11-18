use std::time::Duration;

use derive_more::Constructor;

#[derive(Debug, Clone, Default, Constructor)]
pub struct End {
    pub timeout: Option<Duration>,
}
