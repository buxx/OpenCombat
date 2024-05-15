use battle_core::config::GuiConfig;
use battle_core::config::ServerConfig;
use battle_core::deployment::DeploymentReader;
use battle_core::game::control::MapControl;
use battle_core::map::reader::MapReader;
use battle_core::state::battle::builder::BattleStateBuilder;
use battle_core::utils::start_puffin_server;
use battle_gui::run::run;
use battle_gui::run::Opt;
use battle_gui::GuiError;
use oc_core::resources::Resources;
use structopt::StructOpt;

fn main() -> Result<(), GuiError> {
    let opt = Opt::from_args();
    let map_name: &String = &opt.map_name;
    let resources = Resources::new()?.ensure()?;

    // Profiling server
    if opt.profile {
        start_puffin_server(opt.profile_address.clone())
    };

    let deployment = DeploymentReader::from_file(&opt.deployment)?;
    let a_control = MapControl::new(opt.a_control.clone());
    let b_control = MapControl::new(opt.b_control.clone());

    // TODO : If remote server, download map before read it
    let map = MapReader::new(map_name, &resources.lib())?.build()?;
    let config = GuiConfig::default();
    let server_config = ServerConfig::default();
    let battle_state = BattleStateBuilder::new(map.clone()).build()?;

    run(
        opt.into(),
        config,
        server_config,
        a_control,
        b_control,
        map,
        resources,
        deployment,
        battle_state,
        false,
        vec![],
        vec![],
        vec![],
    )
}
