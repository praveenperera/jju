mod command;
mod sequence;

pub use command::{BindingBehavior, CommandSpec};
pub use sequence::KeySequence;

pub type BindingSpec = CommandSpec;
