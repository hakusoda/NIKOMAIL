use twilight_model::{
	application::interaction::{
		application_command::CommandDataOption,
		InteractionData, InteractionType
	},
	channel::{ Channel, Message },
	guild::Permissions,
	id::{
		marker::{ UserMarker, GuildMarker, ApplicationMarker, InteractionMarker },
		Id
	}
};

#[derive(Clone, Debug, PartialEq)]
pub struct Interaction {
	pub app_permissions: Option<Permissions>,
	pub application_id: Id<ApplicationMarker>,
	pub channel: Option<Channel>,
	pub data: Option<InteractionData>,
	pub guild_id: Option<Id<GuildMarker>>,
	pub guild_locale: Option<String>,
	pub id: Id<InteractionMarker>,
	pub kind: InteractionType,
	pub locale: Option<String>,
	pub message: Option<Message>,
	pub token: String,
	pub user_id: Option<Id<UserMarker>>,
}

impl Interaction {
	pub fn options(&self) -> Vec<&CommandDataOption> {
		match &self.data {
			Some(InteractionData::ApplicationCommand(x)) => x.options.iter().collect(),
			_ => vec![]
		}
	}
	/*pub async fn user(&self) -> Result<Option<Ref<'_, Id<UserMarker>, CachedUser>>> {
		Ok(if let Some(user_id) = self.user_id {
			Some(DISCORD_MODELS.user(user_id).await?)
		} else { None })
	}*/

	/*pub async fn member(&self) -> Result<Option<Ref<'static, (Id<GuildMarker>, Id<UserMarker>), CachedMember>>> {
		Ok(if let Some(user_id) = self.user_id && let Some(guild_id) = self.guild_id {
			Some(CACHE.discord.member(guild_id, user_id).await?)
		} else { None })
	}*/
}