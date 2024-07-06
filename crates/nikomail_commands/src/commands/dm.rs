use nikomail_cache::CACHE;
use nikomail_commands_core::{ Context, Result, command };
use twilight_model::{
	application::command::{ CommandOptionChoice, CommandOptionChoiceValue },
	channel::message::component::{ ActionRow, SelectMenu, SelectMenuOption, SelectMenuType },
	id::Id
};

use crate::util;

async fn autocomplete_topic<'a>(context: Context, partial: String) -> Result<Vec<CommandOptionChoice>> {
	let user_topics = CACHE.nikomail.user_topics(context.author_id().unwrap()).await?;
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
#[command(slash, context = "bot_dm", description = "Close the current or specified topic, you will not be able to reopen it.")]
pub async fn close_topic(
	context: Context,
	#[autocomplete = "autocomplete_topic"]
	#[description = "The topic you wish to close."]
	topic: Option<String>
) -> Result<()> {
	let author_id = context.author_id().unwrap();
	let topic_id = match topic {
		Some(x) => x.parse::<u64>().ok().and_then(Id::new_checked),
		None => CACHE.nikomail.user_state(author_id).await?.current_topic_id
	};
	if let Some(topic_id) = topic_id && util::close_topic(context.interaction.id, &context.interaction.token, topic_id).await? {
		return Ok(());
	}

	let user_topics = CACHE.nikomail
		.user_topics(author_id)
		.await?;
	if !user_topics.is_empty() {
		let mut options: Vec<SelectMenuOption> = Vec::new();
		for topic_id in user_topics.iter() {
			let channel = CACHE.discord
				.channel(*topic_id)
				.await?;
			let label = channel.name.clone().unwrap_or_else(|| "Unknown".into());
			let guild = CACHE.discord
				.guild(channel.guild_id.unwrap())
				.await?;
			options.push(SelectMenuOption {
				default: false,
				description: Some(format!("in {}", guild.name)),
				emoji: None,
				label,
				value: topic_id.to_string()
			});
		}

		return context.reply("Select a topic to close below...")
			.components([ActionRow {
				components: vec![SelectMenu {
					channel_types: None,
					custom_id: "close_topic_menu".into(),
					default_values: None,
					disabled: false,
					kind: SelectMenuType::Text,
					max_values: Some(1),
					min_values: Some(1),
					options: Some(options),
					placeholder: Some("Select a topic...".into())
				}.into()]
			}.into()])
			.ephemeral()
			.await;
	}

	context.reply("You currently have no topics open, silly!")
		.ephemeral()
		.await
}

#[tracing::instrument(skip_all)]
#[command(slash, context = "bot_dm", description = "change the current topic")]
pub async fn set_topic(
	context: Context,
	#[autocomplete = "autocomplete_topic"]
	#[description = "The topic you wish to change to."]
	topic: String
) -> Result<()> {
	if let Ok(int) = topic.parse::<u64>() {
		if let Some(topic_id) = Id::new_checked(int) {
			if CACHE.nikomail.topic(topic_id).await?.is_some() {
				let mut user_state = CACHE.nikomail.user_state_mut(context.author_id().unwrap()).await?;
				user_state.current_topic_id.replace(topic_id);

				return context.reply("success, start talking!")
					.ephemeral()
					.await;
			}
		}
	}

	context.reply("i couldn't find the topic you requested, make sure you're using the options menu!")
		.ephemeral()
		.await
}