#![feature(let_chains)]
pub mod command;
pub mod commands;
pub mod error;
pub mod interaction;
mod macros;
mod util;

pub use error::{ Error, Result };
pub use interaction::Interaction;