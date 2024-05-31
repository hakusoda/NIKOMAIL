use twilight_model::{
	id::Id,
	application::command::{ CommandOptionChoice, CommandOptionChoiceValue }
};
use nikomail_macros::command;

use crate::{
	state::STATE,
	CACHE,
	Result, Context, Interaction, CommandResponse
};
pub async fn autocomplete_topic<'a>(_context: Context, interaction: Interaction, partial: String) -> Result<Vec<CommandOptionChoice>> {
	let user_topics = CACHE.nikomail.user_topics(interaction.user_id.unwrap()).await?;
	let mut choices: Vec<CommandOptionChoice> = vec![];
	for channel_id in user_topics.iter() {
		let channel = CACHE.discord.channel(*channel_id).await?;
		if let Some(name) = &channel.name && name.to_lowercase().starts_with(&partial.to_lowercase()) {
			choices.push(CommandOptionChoice {
				name: name.clone(),
				name_localizations: None,
				value: CommandOptionChoiceValue::String(channel_id.to_string())
			});
		}
	}

	Ok(choices)
}

#[tracing::instrument(skip_all)]
#[command(slash, context = "bot_dm", description = "change the current topic")]
pub async fn set_topic(
	_context: Context,
	interaction: Interaction,
	#[autocomplete = "autocomplete_topic"]
	topic: String
) -> Result<CommandResponse> {
	if let Ok(int) = topic.parse::<u64>() {
		if let Some(topic_id) = Id::new_checked(int) {
			if CACHE.nikomail.topic(topic_id).await?.is_some() {
				let mut user_state = STATE.get().unwrap().user_state(interaction.user_id.unwrap());
				user_state.current_topic_id.replace(topic_id);

				return Ok(CommandResponse::ephemeral("success, start talking!"));
			}
		}
	}
	Ok(CommandResponse::ephemeral("i couldn't find the topic you requested, make sure you're using the options menu!"))
}