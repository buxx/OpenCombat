pub mod bridge;
pub mod examples;
pub mod logging;
pub mod run;
pub mod snapshot;
pub mod tests;
pub mod utils;

use std::sync::OnceLock;

static ARGS: OnceLock<Vec<String>> = OnceLock::new();

/// Command line of the running example: program name followed by the arguments given after the
/// example name. Examples must parse their arguments from this (`Args::parse_from(args())`).
pub fn args() -> Vec<String> {
    ARGS.get_or_init(|| std::env::args().take(1).collect())
        .clone()
}

/// Set the arguments returned by [`args`]. Must be called before any call to [`args`].
pub fn set_args(args: Vec<String>) {
    let _ = ARGS.set(args);
}
