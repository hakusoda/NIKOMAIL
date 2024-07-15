use nikomail_cache::CACHE;
use nikomail_commands_core::Result;
use nikomail_util::{ DISCORD_CLIENT, DISCORD_INTERACTION_CLIENT, PG_POOL };
use std::pin::Pin;
use twilight_model::{
	channel::message::{
		component::{ Button, ActionRow, ButtonStyle, Component },
		MessageFlags, ReactionType
	},
	http::interaction::{ InteractionResponse, InteractionResponseData, InteractionResponseType },
	id::{
		marker::{ ChannelMarker, GuildMarker, InteractionMarker },
		Id
	}
};

pub enum CloseTopicOperation<'a> {
	Generic,
	Deleted(Option<String>),
	Author(Id<InteractionMarker>, &'a str)
}

impl<'a> CloseTopicOperation<'a> {
	pub async fn execute(self, topic_id: Id<ChannelMarker>) -> Result<bool> {
		Ok(if let Some((_,topic)) = CACHE.nikomail.topics.remove(&topic_id) {
			let author_id = topic.author_id;
			let guild_id = topic.server_id;
			let interaction = self.interaction();
			if let Some(interaction) = interaction {
				DISCORD_INTERACTION_CLIENT
					.create_response(interaction.0, interaction.1, &InteractionResponse {
						kind: InteractionResponseType::DeferredChannelMessageWithSource,
						data: Some(InteractionResponseData {
							flags: Some(MessageFlags::EPHEMERAL),
							..Default::default()
						})
					})
					.await?;
			}
	
			CACHE
				.nikomail
				.remove_user_topic(author_id, topic_id);
			CACHE
				.nikomail
				.user_state_mut(author_id)
				.await?
				.current_topic_id = None;
	
			sqlx::query!(
				"
				DELETE from topics
				WHERE id = $1
				",
				topic_id.get() as i64
			)
				.execute(&*Pin::static_ref(&PG_POOL).await)
				.await?;
	
			if let Some(interaction) = interaction {
				DISCORD_CLIENT
					.update_thread(topic_id)
					.locked(true)
					.archived(true)
					.await?;

				DISCORD_CLIENT
					.create_message(topic_id)
					.content("# Topic has been closed\nThe author of this topic has closed the topic, it cannot be reopened.\nMessages past this point will not be sent, feel free to delete this thread if necessary.")
					.await?;

				DISCORD_INTERACTION_CLIENT
					.update_response(interaction.1)
					.content(Some("The topic has been closed, it cannot be reopened, feel free to open another one!"))
					.components(Some(&[create_topic_button(Some(guild_id)).await?]))
					.await?;
			} else {
				let private_channel_id = CACHE
					.discord
					.private_channel(author_id)
					.await?;
				let channel_name = self
					.channel_name(topic_id)
					.await?;
				let guild = CACHE
					.discord
					.guild(guild_id)
					.await?;
				DISCORD_CLIENT.create_message(private_channel_id)
					.content(&format!("## Your topic in {} has been closed\n**{channel_name}** has been closed by server staff, feel free to open another one!", guild.name))
					.components(&[create_topic_button(Some(guild_id)).await?])
					.await?;
			}
	
			true
		} else { false })
	}

	async fn channel_name(&self, channel_id: Id<ChannelMarker>) -> Result<String> {
		Ok(match self {
			CloseTopicOperation::Deleted(channel_name) => channel_name.clone(),
			_ => CACHE
				.discord
				.channel(channel_id)
				.await?
				.name
				.clone()
		}.unwrap_or_else(|| "Unknown".into()))
	}

	fn interaction(&self) -> Option<(Id<InteractionMarker>, &str)> {
		match self {
			CloseTopicOperation::Author(x, y) => Some((*x, y)),
			_ => None
		}
	}
}

pub async fn create_topic_button(guild_id: Option<Id<GuildMarker>>) -> Result<Component> {
	Ok(ActionRow {
		components: vec![
			Button {
				url: None,
				label: Some(match guild_id {
					Some(guild_id) => format!("Start new topic in {}", CACHE.discord.guild(guild_id).await?.name),
					None => "Start new topic".into()
				}),
				emoji: Some(ReactionType::Custom { animated: false, id: Id::new(1219234152709095424), name: Some("dap_me_up".into()) }),
				style: ButtonStyle::Primary,
				disabled: false,
				custom_id: Some(match guild_id {
					Some(guild_id) => format!("create_topic_{guild_id}"),
					None => "create_topic".into()
				})
			}.into()
		]
	}.into())
}