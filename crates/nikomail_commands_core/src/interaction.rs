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
}