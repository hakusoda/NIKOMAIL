#![feature(let_chains, impl_trait_in_assoc_type)]
pub mod command;
pub mod context;
pub mod error;
pub mod interaction;
pub mod macros;
mod util;

pub use command::Command;
pub use context::Context;
pub use error::*;
pub use interaction::Interaction;
pub use nikomail_commands_core_macros::*;