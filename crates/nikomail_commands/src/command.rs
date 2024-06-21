use serde::Serialize;
use serde_repr::Serialize_repr;
use nikomail_util::DISCORD_INTERACTION_CLIENT;
use twilight_model::{
	application::command::CommandOptionChoice,
	channel::message::{ Component, MessageFlags },
	guild::Permissions
};

use crate::{
	util::{ BoxFuture, serialize_option_as_bool },
	Error, Result, Interaction
};

pub struct Command {
	pub name: String,
	pub options: Vec<CommandOption>,
	pub contexts: Vec<CommandContext>,
	pub handler: fn(Interaction) -> BoxFuture<'static, Result<CommandResponse>>,
	pub is_user: bool,
	pub is_slash: bool,
	pub is_message: bool,
	pub description: Option<String>,
	pub default_member_permissions: Option<u64>
}

impl Command {
	pub fn default_member_permissions(&self) -> Option<Permissions> {
		self.default_member_permissions.map(Permissions::from_bits_truncate)
	}
}

#[derive(Clone, Serialize_repr)]
#[repr(u8)]
pub enum CommandContext {
	Guild,
	BotDM,
	PrivateChannel
}

#[derive(Clone, Serialize)]
pub struct CommandOption {
	pub name: String,
	#[serde(rename = "type")]
	pub kind: CommandOptionKind,
	pub required: bool,
	pub description: Option<String>,
	#[serde(serialize_with = "serialize_option_as_bool")]
	#[allow(clippy::type_complexity)]
	pub autocomplete: Option<fn(Interaction, String) -> BoxFuture<'static, Result<Vec<CommandOptionChoice>>>>
}

#[derive(Clone, Serialize_repr)]
#[repr(u8)]
pub enum CommandOptionKind {
	SubCommand = 1,
	SubCommandGroup,
	String,
	Integer,
	Boolean,
	User,
	Channel,
	Role,
	Mentionable,
	Number,
	Attachment
}

pub enum CommandResponse {
	Message {
		flags: Option<MessageFlags>,
		content: Option<String>,
		components: Option<Vec<Component>>
	},
	Defer
}

impl CommandResponse {
	pub fn defer(interaction_token: impl Into<String>, callback: BoxFuture<'static, Result<()>>) -> Self {
		let interaction_token = interaction_token.into();
		tokio::spawn(async move {
			if let Err(error) = callback.await {
				tracing::error!("error during interaction: {}", error);
				let (text, problem) = match error {
					Error::TwilightHttp(error) => (" while communicating with discord...", error.to_string()),
					_ => (", not sure what exactly though!", error.to_string())
				};
				DISCORD_INTERACTION_CLIENT.update_response(&interaction_token)
					.content(Some(&format!("<:niko_look_left:1227198516590411826> something unexpected happened{text}\n```diff\n- {problem}```")))
					.await
					.unwrap();
			}
		});
		Self::Defer
	}

	pub fn ephemeral(content: impl Into<String>) -> Self {
		Self::Message {
			flags: Some(MessageFlags::EPHEMERAL),
			content: Some(content.into()),
			components: None
		}
	}
}