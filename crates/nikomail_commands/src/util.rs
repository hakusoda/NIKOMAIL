use nikomail_cache::CACHE;
use nikomail_commands_core::Result;
use nikomail_util::{ DISCORD_CLIENT, DISCORD_INTERACTION_CLIENT, PG_POOL };
use twilight_model::{
	channel::message::{
		component::{ Button, ActionRow, ButtonStyle, Component },
		ReactionType
	},
	http::interaction::{ InteractionResponse, InteractionResponseType },
	id::{
		marker::{ ChannelMarker, GuildMarker, InteractionMarker },
		Id
	}
};

pub async fn close_topic(interaction_id: Id<InteractionMarker>, interaction_token: &str, topic_id: Id<ChannelMarker>) -> Result<bool> {
	Ok(if let Some(topic) = CACHE.nikomail.topic_mut(topic_id).await?.take() {
		let author_id = topic.author_id;
		let guild_id = topic.server_id;
		DISCORD_INTERACTION_CLIENT
			.create_response(interaction_id, interaction_token, &InteractionResponse {
				kind: InteractionResponseType::DeferredChannelMessageWithSource,
				data: None
			})
			.await?;

		sqlx::query!(
			"
			DELETE from topics
			WHERE id = $1
			",
			topic_id.get() as i64
		)
			.execute(&*std::pin::Pin::static_ref(&PG_POOL).await)
			.await?;

		let mut user_state = CACHE.nikomail.user_state_mut(author_id).await?;
		user_state.current_topic_id = None;

		DISCORD_CLIENT.create_message(topic_id)
			.content("# Topic has been closed\nThe author of this topic has closed the topic, it cannot be reopened.\nMessages past this point will not be sent, feel free to delete this thread if necessary.")
			.await?;

		DISCORD_CLIENT.update_thread(topic_id)
			.locked(true)
			.archived(true)
			.await?;

		CACHE.nikomail.remove_user_topic(author_id, topic_id);

		DISCORD_INTERACTION_CLIENT
			.update_response(interaction_token)
			.content(Some("The topic has been closed, it cannot be reopened, feel free to open another one!"))
			.components(Some(&[create_topic_button(Some(guild_id))]))
			.await?;

		true
	} else { false })
}

pub fn create_topic_button(guild_id: Option<Id<GuildMarker>>) -> Component {
	ActionRow {
		components: vec![
			Button {
				url: None,
				label: Some("Start new topic".into()),
				emoji: Some(ReactionType::Custom { animated: false, id: Id::new(1219234152709095424), name: Some("dap_me_up".into()) }),
				style: ButtonStyle::Primary,
				disabled: false,
				custom_id: Some(match guild_id {
					Some(x) => format!("create_topic_{x}"),
					None => "create_topic".into()
				})
			}.into()
		]
	}.into()
}