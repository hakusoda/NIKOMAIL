use once_cell::sync::Lazy;

use crate::Command;

pub mod dm;
pub mod guild;

pub static COMMANDS: Lazy<Vec<Command>> = Lazy::new(|| vec![
	dm::set_topic(),
	dm::close_topic(),
	guild::create_button(),
	guild::blacklist_topic_author()
]);