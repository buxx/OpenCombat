mod entity;
mod message;
mod orchestrator;
mod types;

fn main() {
    let entities = vec![];
    let orchestrator = orchestrator::Orchestrator::new(entities);
    orchestrator.operate();
}
