#![feature(let_chains, try_blocks, const_async_blocks, type_alias_impl_trait)]
use std::pin::Pin;
use clap::Parser;
use serde::Serialize;
use tracing::{ Level, info };
use once_cell::sync::Lazy;
use nikomail_cache::CACHE;
use nikomail_commands::commands::COMMANDS;
use nikomail_commands_core::command::{ Command, CommandContext, CommandOption, CommandOptionKind };
use nikomail_models::nikomail::RelayedMessageModel;
use nikomail_util::{ DISCORD_APP_ID, DISCORD_CLIENT, DISCORD_INTERACTION_CLIENT, PG_POOL };
use twilight_model::{
	guild::Permissions,
	application::command::CommandType
};
use twilight_gateway::CloseFrame;
use tracing_subscriber::FmtSubscriber;

mod discord;
mod error;
mod util;

pub use error::Result;

#[derive(Parser)]
struct Args {
	#[clap(long, short)]
    update_commands: bool
}

#[derive(Serialize)]
struct ApplicationCommand {
	#[serde(rename = "type")]
	kind: CommandType,
	name: String,
	options: Vec<CommandOption>,
	contexts: Vec<CommandContext>,
	description: String,
	default_member_permissions: Option<Permissions>
}

fn app_command(command: &Command, kind: CommandType) -> Result<ApplicationCommand> {
	let description = match kind {
		CommandType::User => "",
		_ => command.description.as_ref().map_or("there is no description yet, how sad...", |x| x.as_str())
	};
	let mut options = command.options.clone();
	for subcommand in command.subcommands.iter() {
		options.push(CommandOption {
			kind: CommandOptionKind::SubCommand,
			name: subcommand.name.clone(),
			required: false,
			description: subcommand.description.clone().or(Some("there is no description yet, how sad...".into())),
			autocomplete: None,
			channel_kinds: None,
			options: subcommand.options.clone()
		});
	}

	Ok(ApplicationCommand {
		kind,
		name: command.name.clone(),
		options,
		contexts: command.contexts.clone(),
		description: description.to_string(),
		default_member_permissions: command.default_member_permissions()
	})
}

#[tokio::main(flavor = "multi_thread")]
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
		for command in COMMANDS.iter() {
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
				.build()
				.unwrap()
		).await.unwrap();

		info!("successfully updated global commands");
	} else {
		Lazy::force(&CACHE);
		Lazy::force(&COMMANDS);
		Lazy::force(&DISCORD_INTERACTION_CLIENT); // also evaluates DISCORD_CLIENT & DISCORD_APP_ID
		Pin::static_ref(&discord::DISCORD_APP_COMMANDS).await;

		let relayed_messages = RelayedMessageModel::get_all().await.unwrap();
		info!("fetched {} relayed messages", relayed_messages.len());

		for relayed_message in relayed_messages {
			CACHE.nikomail.add_relayed_message(relayed_message);
		}

		let message_sender = discord::gateway::initialise();
		tokio::signal::ctrl_c().await.unwrap();

		info!("shutdown signal received, saving stuff...");
		
		message_sender.close(CloseFrame::NORMAL).unwrap();

		let mut transaction = Pin::static_ref(&PG_POOL).await.begin().await.unwrap();
		for user_state in CACHE.nikomail.user_states.iter() {
			sqlx::query!(
				"
				INSERT INTO user_states
				VALUES ($1, $2)
				ON CONFLICT (id)
				DO UPDATE SET current_topic_id = $2
				",
				user_state.id.get() as i64,
				user_state.current_topic_id.map(|x| x.get() as i64)
			)
				.execute(&mut *transaction)
				.await
				.unwrap();
		}

		transaction.commit().await.unwrap();
		info!("saved {} user states, now shutting down...", CACHE.nikomail.user_states.len());
	}
}