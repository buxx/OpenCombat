use std::{path::PathBuf, process::ExitCode};

use clap::Parser;
use oc_world::check::check_map;

#[derive(Parser, Debug, Clone)]
#[command(version, about = "Check world map against mod", long_about = None)]
struct Args {
    /// Path to the mod folder
    #[clap()]
    r#mod: PathBuf,

    /// Path to the world (map) folder
    #[clap()]
    map: PathBuf,
}

fn main() -> ExitCode {
    let args = Args::parse();
    let errors = check_map(&args.r#mod, &args.map);

    if errors.is_empty() {
        println!("OK");
        return ExitCode::SUCCESS;
    }

    for error in &errors {
        eprintln!("{error}");
    }

    eprintln!("{} error(s)", errors.len());
    ExitCode::FAILURE
}
