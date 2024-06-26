use nikomail_cache::CACHE;
use nikomail_util::{ PG_POOL, DISCORD_CLIENT, DISCORD_INTERACTION_CLIENT };
use twilight_model::{
	id::Id,
	application::command::{ CommandOptionChoice, CommandOptionChoiceValue }
};
use nikomail_macros::command;

use crate::{
	command::CommandResponse,
	util::create_topic_button,
	Result, Interaction
};
async fn autocomplete_topic<'a>(interaction: Interaction, partial: String) -> Result<Vec<CommandOptionChoice>> {
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
#[command(slash, context = "bot_dm", description = "Close the current or specified topic, you will not be able to reopen it.")]
pub async fn close_topic(
	interaction: Interaction,
	#[autocomplete = "autocomplete_topic"]
	topic: Option<String>
) -> Result<CommandResponse> {
	let user_id = interaction.user_id.unwrap();
	let topic_id = match topic {
		Some(x) => x.parse::<u64>().ok().and_then(Id::new_checked),
		None => CACHE.nikomail.user_state(user_id).await?.current_topic_id
	};
	if let Some(topic_id) = topic_id && let Some(topic) = CACHE.nikomail.topic_mut(topic_id).await?.take() {
		return Ok(CommandResponse::defer(interaction.token.clone(), Box::pin(async move {
			let mut user_state = CACHE.nikomail.user_state_mut(interaction.user_id.unwrap()).await?;
			user_state.current_topic_id = None;

			DISCORD_CLIENT.create_message(topic_id)
				.content("# Topic has been closed\nThe author of this topic has closed the topic, it cannot be reopened.\nMessages past this point will not be sent, feel free to delete this thread if necessary.")
				.await?;

			DISCORD_CLIENT.update_thread(topic_id)
				.locked(true)
				.archived(true)
				.await?;

			CACHE.nikomail.remove_user_topic(user_id, topic_id);

			sqlx::query!(
				"
				DELETE from topics
				WHERE id = $1
				",
				topic_id.get() as i64
			)
				.execute(&*std::pin::Pin::static_ref(&PG_POOL).await)
				.await?;

			DISCORD_INTERACTION_CLIENT.update_response(&interaction.token)
				.content(Some("The topic has been closed, it cannot be reopened, feel free to open another one!"))
				.components(Some(&[create_topic_button(Some(topic.server_id))]))
				.await?;
			Ok(())
		})));
	}
	Ok(CommandResponse::ephemeral("i couldn't find the topic you requested!"))
}

#[tracing::instrument(skip_all)]
#[command(slash, context = "bot_dm", description = "change the current topic")]
pub async fn set_topic(
	interaction: Interaction,
	#[autocomplete = "autocomplete_topic"]
	topic: String
) -> Result<CommandResponse> {
	if let Ok(int) = topic.parse::<u64>() {
		if let Some(topic_id) = Id::new_checked(int) {
			if CACHE.nikomail.topic(topic_id).await?.is_some() {
				let mut user_state = CACHE.nikomail.user_state_mut(interaction.user_id.unwrap()).await?;
				user_state.current_topic_id.replace(topic_id);

				return Ok(CommandResponse::ephemeral("success, start talking!"));
			}
		}
	}
	Ok(CommandResponse::ephemeral("i couldn't find the topic you requested, make sure you're using the options menu!"))
}