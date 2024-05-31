#![feature(let_chains, try_blocks)]
#![recursion_limit = "256"]
use std::sync::Arc;
use clap::Parser;
use sqlx::PgPool;
use serde::{ Serialize, Serializer };
use tracing::{ Level, info };
use futures::future::BoxFuture;
use mimalloc::MiMalloc;
use once_cell::sync::Lazy;
use serde_repr::Serialize_repr;
use nikomail_util::{ PG_POOL, DISCORD_APP_ID, DISCORD_CLIENT };
use nikomail_cache::Cache;
use twilight_model::{
	id::{
		marker::{ GuildMarker, ChannelMarker },
		Id
	},
	guild::Permissions,
	channel::message::{ Component, MessageFlags },
	application::{
		command::{ CommandType, CommandOptionChoice },
		interaction::application_command::CommandOptionValue
	}
};
use tracing_subscriber::FmtSubscriber;

pub mod error;
pub mod state;
pub mod discord;

use error::ErrorKind;
use discord::{
	interactions::Interaction,
	INTERACTION
};
pub use error::Result;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub type Context = Arc<discord::gateway::Context>;

pub static CACHE: Lazy<Cache> = Lazy::new(Cache::default);

pub struct Command {
	name: String,
	options: Vec<CommandOption>,
	contexts: Vec<InteractionContextKind>,
	handler: fn(Context, Interaction) -> BoxFuture<'static, Result<CommandResponse>>,
	is_user: bool,
	is_slash: bool,
	is_message: bool,
	description: Option<String>,
	default_member_permissions: Option<String>
}

impl Command {
	pub fn default_member_permissions(&self) -> Result<Option<Permissions>> {
		Ok(if let Some(permissions) = self.default_member_permissions.as_ref() {
			Some(Permissions::from_bits_truncate(permissions.parse()?))
		} else { None })
	}
}

#[derive(Clone, Serialize)]
pub struct CommandOption {
	name: String,
	#[serde(rename = "type")]
	kind: CommandOptionKind,
	required: bool,
	description: Option<String>,
	#[serde(serialize_with = "serialize_option_as_bool")]
	#[allow(clippy::type_complexity)]
	autocomplete: Option<fn(Context, Interaction, String) -> BoxFuture<'static, Result<Vec<CommandOptionChoice>>>>
}

fn serialize_option_as_bool<T, S: Serializer>(value: &Option<T>, serialiser: S) -> core::result::Result<S::Ok, S::Error> {
	serialiser.serialize_bool(value.is_some())
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

pub enum CommandKind {
	Slash,
	User,
	Message
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
				let (text, problem) = match &error.kind {
					ErrorKind::TwilightHttpError(error) => (" while communicating with discord...", error.to_string()),
					_ => (", not sure what exactly though!", error.to_string())
				};
				INTERACTION.update_response(&interaction_token)
					.content(Some(&format!("<:niko_look_left:1227198516590411826> something unexpected happened{text}\n```diff\n- {problem}\n--- {}```", error.context))).unwrap()
					.await.unwrap();
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

#[derive(Parser)]
struct Args {
	#[clap(long, short)]
    update_commands: bool
}

#[derive(Clone, Serialize_repr)]
#[repr(u8)]
enum InteractionContextKind {
	Guild,
	BotDM,
	PrivateChannel
}

#[derive(Serialize)]
struct ApplicationCommand {
	#[serde(rename = "type")]
	kind: CommandType,
	name: String,
	options: Vec<CommandOption>,
	contexts: Vec<InteractionContextKind>,
	description: String,
	default_member_permissions: Option<Permissions>
}

fn app_command(command: &Command, kind: CommandType) -> Result<ApplicationCommand> {
	let description = match kind {
		CommandType::User => "",
		_ => command.description.as_ref().map_or("there is no description yet, how sad...", |x| x.as_str())
	};
	Ok(ApplicationCommand {
		kind,
		name: command.name.clone(),
		options: command.options.iter().map(|x| CommandOption {
			description: x.description.clone().or(Some("there is no description yet, how sad...".into())),
			..x.clone()
		}).collect(),
		contexts: command.contexts.clone(),
		description: description.to_string(),
		default_member_permissions: command.default_member_permissions()?
	})
}

#[tokio::main]
async fn main() {
	let subscriber = FmtSubscriber::builder()
		.with_max_level(Level::INFO)
		.finish();

	tracing::subscriber::set_global_default(subscriber)
		.expect("setting default subscriber failed");

	info!("starting NIKOMAIL v{}", env!("CARGO_PKG_VERSION"));
	
	let args = Args::parse();
	if args.update_commands {
		let mut commands: Vec<ApplicationCommand> = vec![];
		for command in discord::commands::COMMANDS.iter() {
			if command.is_user {
				commands.push(app_command(command, CommandType::User).unwrap());
			}
			if command.is_slash {
				commands.push(app_command(command, CommandType::ChatInput).unwrap());
			}
			if command.is_message {
				commands.push(app_command(command, CommandType::Message).unwrap());
			}
		}

		DISCORD_CLIENT.request::<()>(
			twilight_http::request::Request::builder(&twilight_http::routing::Route::SetGlobalCommands {
				application_id: DISCORD_APP_ID.get()
			})
				.json(&commands)
				.map(twilight_http::request::RequestBuilder::build)
				.unwrap()
		).await.unwrap();

		info!("successfully updated global commands");
	}

	PG_POOL.set(PgPool::connect(env!("DATABASE_URL"))
		.await
		.unwrap()
	).unwrap();

	state::STATE.set(state::State::default()).unwrap();

	discord::gateway::initialise().await;
}

#[macro_export]
macro_rules! parse_command_argument {
    // extracts #[choices(...)]
    /*($interaction:ident, $args:ident => $name:literal: INLINE_CHOICE $type:ty [$($index:literal: $value:literal),*]) => {
        if let Some(arg) = $args.iter().find(|arg| arg.name == $name) {
            let $crate::serenity_prelude::ResolvedValue::Integer(index) = arg.value else {
                return Err($crate::SlashArgError::new_command_structure_mismatch("expected integer, as the index for an inline choice parameter"));
            };
            match index {
                $( $index => $value, )*
                _ => return Err($crate::SlashArgError::new_command_structure_mismatch("out of range index for inline choice parameter")),
            }
        } else {
            return Err($crate::SlashArgError::new_command_structure_mismatch("a required argument is missing"));
        }
    };*/

    // extracts Option<T>
    ($interaction:ident, $args:ident => $name:literal: Option<$type:ty $(,)*>) => {
        if let Some(arg) = $args.iter().find(|arg| arg.name == $name) {
            Some($crate::extract_command_argument!($type, $interaction, &arg.value).await?)
        } else {
            None
        }
    };

    // extracts Vec<T>
    ($interaction:ident, $args:ident => $name:literal: Vec<$type:ty $(,)*>) => {
        match $crate::parse_command_argument!($interaction, $args => $name: Option<$type>) {
            Some(value) => vec![value],
            None => vec![],
        }
    };

    // extracts #[flag]
    ($interaction:ident, $args:ident => $name:literal: FLAG) => {
        $crate::parse_command_argument!($interaction, $args => $name: Option<bool>)
            .unwrap_or(false)
    };

    // exracts T
    ($interaction:ident, $args:ident => $name:literal: $($type:tt)*) => {
        $crate::parse_command_argument!($interaction, $args => $name: Option<$($type)*>).unwrap()
    };
}

#[macro_export]
macro_rules! parse_command_arguments {
    ($interaction:expr, $args:expr => $(
        ( $name:literal: $($type:tt)* )
    ),* $(,)? ) => {
        async {
			let (interaction, args) = ($interaction, $args);
            Ok::<_, $crate::error::Error>(( $(
                $crate::parse_command_argument!( interaction, args => $name: $($type)* ),
            )* ))
        }
    };
}

#[derive(Debug)]
pub enum SlashArgError {
    CommandStructureMismatch {
        description: &'static str
    },
    Parse {
        error: Box<dyn std::error::Error + Send + Sync>,
        input: String
    },
    Invalid(&'static str),
    __NonExhaustive
}

impl SlashArgError {
    pub fn new_command_structure_mismatch(description: &'static str) -> Self {
        Self::CommandStructureMismatch { description }
    }
}

impl std::fmt::Display for SlashArgError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::CommandStructureMismatch { description } => {
				write!(
					f,
					"Bot author did not register their commands correctly ({description})",
				)
			}
			Self::Parse { error, input } => {
				write!(f, "Failed to parse `{input}` as argument: {error}")
			}
			Self::Invalid(description) => {
				write!(f, "You can't use this parameter here: {description}",)
			}
			Self::__NonExhaustive => unreachable!(),
		}
	}
}
impl std::error::Error for SlashArgError {
	fn cause(&self) -> Option<&dyn std::error::Error> {
		match self {
			Self::Parse { error, input: _ } => Some(&**error),
			Self::Invalid { .. } | Self::CommandStructureMismatch { .. } => None,
			Self::__NonExhaustive => unreachable!(),
		}
	}
}

#[async_trait::async_trait]
pub trait CommandArgumentExtractor<T>: Sized {
	async fn extract(
		self,
		interaction: &Interaction,
		value: &CommandOptionValue
	) -> Result<T>;
}

#[async_trait::async_trait]
impl<T: ArgumentConvert + Send + Sync> CommandArgumentExtractor<T> for std::marker::PhantomData<T> {
	async fn extract(
		self,
		interaction: &Interaction,
		value: &CommandOptionValue
	) -> Result<T> {
		T::convert(
			interaction.guild_id,
			interaction.channel.as_ref().map(|x| x.id),
			value
		).await
	}
}

#[async_trait::async_trait]
trait ArgumentConvert: Sized {
	async fn convert(guild_id: Option<Id<GuildMarker>>, channel_id: Option<Id<ChannelMarker>>, value: &CommandOptionValue) -> Result<Self>;
}

#[async_trait::async_trait]
impl ArgumentConvert for String {
	async fn convert(_guild_id: Option<Id<GuildMarker>>, _channel_id: Option<Id<ChannelMarker>>, value: &CommandOptionValue) -> Result<Self> {
		match value {
			CommandOptionValue::String(x) => Ok(x.clone()),
			_ => Err(error::ErrorKind::Unknown.into())
		}
	}
}

#[macro_export]
macro_rules! extract_command_argument {
	($target:ty, $interaction:expr, $value:expr) => {{
		use $crate::CommandArgumentExtractor as _;
		(&&std::marker::PhantomData::<$target>).extract(&$interaction, $value)
	}};
}