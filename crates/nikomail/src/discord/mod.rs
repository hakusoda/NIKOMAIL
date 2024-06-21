use std::pin::Pin;
use nikomail_util::DISCORD_INTERACTION_CLIENT;
use twilight_model::{
	id::{ marker::CommandMarker, Id },
	application::command::Command
};
use async_once_cell::Lazy;

pub mod gateway;
pub mod interactions;

pub type CommandsFuture = impl Future<Output = Vec<Command>> + Send;
pub static DISCORD_APP_COMMANDS: Lazy<Vec<Command>, CommandsFuture> = Lazy::new(async {
	DISCORD_INTERACTION_CLIENT.global_commands().await.unwrap().model().await.unwrap()
});

pub async fn app_command_id(name: &str) -> Option<Id<CommandMarker>> {
	Pin::static_ref(&DISCORD_APP_COMMANDS)
		.await
		.iter()
		.find(|x| x.name == name)
		.and_then(|x| x.id)
}