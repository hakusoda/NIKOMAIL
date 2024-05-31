use once_cell::sync::Lazy;

use crate::Command;

pub mod dm;
pub mod test;
pub mod guild;

pub static COMMANDS: Lazy<Vec<Command>> = Lazy::new(|| vec![
	dm::set_topic(),
	guild::create_button()
]);