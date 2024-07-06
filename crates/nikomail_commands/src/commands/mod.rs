use nikomail_commands_core::{ Command, Context };
use once_cell::sync::Lazy;
use twilight_model::application::interaction::{
	application_command::CommandOptionValue,
	InteractionData
};

mod dm;
mod guild;

pub static COMMANDS: Lazy<Vec<Command>> = Lazy::new(|| vec![
	dm::close_topic(),
	dm::set_topic(),

	guild::configure(),
	guild::create_button(),
	guild::blacklist_topic_author()
]);

pub fn process_context(mut context: Context) -> Option<(Context, &'static Command)> {
	let interaction = &context.interaction;
	if
		let Some(InteractionData::ApplicationCommand(command_data)) = &interaction.data &&
		let Some(command) = COMMANDS.iter().find(|x| x.name == command_data.name)
	{
		if command.subcommands.is_empty() {
			return Some((context, command));
		} else if // this currently doesn't support subcommand categories or deeply nested subcommands
			let Some(option) = command_data.options.first() &&
			let CommandOptionValue::SubCommand(options) = &option.value &&
			let Some(subcommand) = command.subcommands.iter().find(|x| x.name == option.name)
		{
			context.options.clone_from(options);
			return Some((context, subcommand));
		}
	}

	None
}