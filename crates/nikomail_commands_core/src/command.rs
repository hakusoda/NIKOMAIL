use serde::Serialize;
use serde_repr::Serialize_repr;
use twilight_model::{
	application::command::CommandOptionChoice,
	channel::ChannelType,
	guild::Permissions
};

use crate::{
	util::{ BoxFuture, serialize_option_as_bool },
	Context, CoreError
};

pub struct Command {
	pub name: String,
	pub options: Vec<CommandOption>,
	pub contexts: Vec<CommandContext>,
	pub handler: fn(Context) -> BoxFuture<'static, Result<(), CoreError>>,
	pub is_user: bool,
	pub is_slash: bool,
	pub is_message: bool,
	pub description: Option<String>,
	pub default_member_permissions: Option<u64>,
	pub subcommands: Vec<Command>
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
	pub autocomplete: Option<fn(Context, String) -> BoxFuture<'static, Result<Vec<CommandOptionChoice>, CoreError>>>,
	#[serde(rename = "channel_types")]
	pub channel_kinds: Option<Vec<ChannelType>>,
	pub options: Vec<CommandOption>
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