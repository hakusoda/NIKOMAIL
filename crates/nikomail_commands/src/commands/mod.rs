use once_cell::sync::Lazy;

use crate::command::Command;

mod dm;
mod guild;

pub static COMMANDS: Lazy<Vec<Command>> = Lazy::new(|| vec![
	dm::close_topic(),
	dm::set_topic(),

	guild::create_button(),
	guild::blacklist_topic_author()
]);