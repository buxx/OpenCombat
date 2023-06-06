use std::{fs::File, io::Write, path::PathBuf};

use battle_tools::hardcode;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct Opt {
    #[structopt(default_value = "./demo1_deployment.json", parse(from_os_str))]
    output: PathBuf,
}
fn main() {
    let opt = Opt::from_args();
    let deployment = serde_json::to_string(&hardcode::demo1_deployment()).unwrap();
    let mut output = File::create(opt.output).unwrap();
    output.write_all(&deployment.as_bytes()).unwrap();
}
