//! Single binary grouping all examples: `oc_example <name> [example args...]`.
use clap::{CommandFactory, Parser, ValueEnum};

use oc_examples::examples as e;

#[derive(Parser, Debug)]
#[command(about = "Run one of the examples", long_about = None, disable_help_flag = true, disable_version_flag = true)]
struct Cli {
    /// Example to run (`-h` to list them)
    example: Option<Example>,
    /// Arguments given to the example (use `--help` after the name to list them)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
enum Example {
    AiFight,
    Demo2,
    Engaging,
    Height,
    IndividualBehaviors,
    IndividualShots,
    Interiors,
    Minidblue,
    ProjectilesObstacles,
    Regions,
    Stress,
    StressProjectiles,
    StressProjectilesServer,
    Visibilities,
    Weapons,
    World,
    World1,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if matches!(std::env::args().nth(1).as_deref(), Some("-h" | "--help")) {
        Cli::command().print_help()?;
        return Ok(());
    }

    let cli = Cli::parse();
    // Help flags only concern this launcher when no example is given: otherwise the example
    // handles them (`oc_example world --help`).
    let Some(example) = cli.example else {
        Cli::command().print_help()?;
        return Ok(());
    };
    let name = example
        .to_possible_value()
        .map(|v| v.get_name().to_string())
        .unwrap_or_default();

    // Examples parse their own arguments: give them "<bin> <name>" followed by their arguments
    let mut args = vec![format!("{} {name}", "oc_example")];
    args.extend(cli.args);
    oc_examples::set_args(args);

    match example {
        Example::AiFight => e::ai_fight::main(),
        Example::Demo2 => e::demo2::main(),
        Example::Engaging => e::engaging::main().map_err(Into::into),
        Example::Height => e::height::main(),
        Example::IndividualBehaviors => e::individual_behaviors::main(),
        Example::IndividualShots => e::individual_shots::main(),
        Example::Interiors => e::interiors::main().map_err(Into::into),
        Example::Minidblue => e::minidblue::main(),
        Example::ProjectilesObstacles => e::projectiles_obstacles::main(),
        Example::Regions => e::regions::main().map_err(Into::into),
        Example::Stress => e::stress::main(),
        Example::StressProjectiles => e::stress_projectiles::main(),
        Example::StressProjectilesServer => e::stress_projectiles_server::main(),
        Example::Visibilities => e::visibilities::main().map_err(Into::into),
        Example::Weapons => e::weapons::main().map_err(Into::into),
        Example::World => e::world::main(),
        Example::World1 => e::world1::main(),
    }
}
