// WARNING: ai modue is generated and modified by AI
// To see ai behavior, us ai-fight example:
//     OC_SPEED=100 cargo run -p oc_examples --features debug --bin oc_example -- ai-fight --a-ai offensive --b-ai defensive --map demo2 --see-all-enemies --auto-fight
//! Rule based AIs able to play one side of a battle (see [`AiKind`]).
pub mod capture;
pub mod combat;
pub mod command;
pub mod defensive;
pub mod kind;
pub mod offensive;
pub mod perception;
pub mod support;

pub use command::Command;
pub use defensive::Defensive;
pub use kind::AiKind;
pub use offensive::Offensive;
pub use perception::Perception;

/// AI playing one side: decides commands from what the side knows
pub trait Commander: Send {
    /// Commands to apply now (called regularly by the server)
    fn tick(&mut self, perception: &Perception) -> Vec<Command>;
}
