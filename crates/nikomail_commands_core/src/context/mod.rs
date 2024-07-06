use nikomail_util::DISCORD_INTERACTION_CLIENT;
use twilight_http::request::application::interaction::UpdateResponse;
use twilight_model::{
	application::interaction::application_command::CommandDataOption,
	channel::message::MessageFlags,
	http::interaction::{ InteractionResponse, InteractionResponseType },
	id::{ marker::{ ChannelMarker, GuildMarker, UserMarker }, Id }
};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::{ Interaction, Result };

pub mod reply_builder;
pub use reply_builder::ReplyBuilder;

pub struct Context {
	pub interaction: Interaction,
	pub options: Vec<CommandDataOption>
}

impl Context {
	pub fn new(interaction: Interaction) -> Self {
		let options = interaction.options().into_iter().cloned().collect();
		Self {
			interaction,
			options
		}
	}

	pub fn author_id(&self) -> Option<Id<UserMarker>> {
		self.interaction.user_id
	}

	pub fn channel_id(&self) -> Option<Id<ChannelMarker>> {
		self.interaction.channel.as_ref().map(|x| x.id)
	}

	pub fn guild_id(&self) -> Option<Id<GuildMarker>> {
		self.interaction.guild_id
	}

	pub async fn defer(&self) -> Result<()> {
		self.response(InteractionResponse {
			kind: InteractionResponseType::DeferredChannelMessageWithSource,
			data: Some(
				InteractionResponseDataBuilder::new()
					.flags(MessageFlags::EPHEMERAL)
					.build()
			)
		}).await?;

		Ok(())
	}

	pub fn update(&self) -> UpdateResponse<'_> {
		DISCORD_INTERACTION_CLIENT.update_response(&self.interaction.token)
	}

	pub fn reply(&self, content: impl Into<String>) -> ReplyBuilder {
		/*let mut builder = InteractionResponseDataBuilder::new()
			.content(text);
		if ephemeral {
			builder = builder.flags(MessageFlags::EPHEMERAL);
		}

		self.response(InteractionResponse {
			kind: InteractionResponseType::ChannelMessageWithSource,
			data: Some(builder.build())
		}).await*/
		ReplyBuilder::new(self, content)
	}

	pub async fn response(&self, response: InteractionResponse) -> Result<()> {
		DISCORD_INTERACTION_CLIENT.create_response(
			self.interaction.id,
			&self.interaction.token,
			&response
		).await?;

		Ok(())
	}
}