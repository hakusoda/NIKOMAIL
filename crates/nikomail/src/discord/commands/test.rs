use nikomail_macros::command;

use crate::{
	Result, Context, Interaction, CommandResponse
};

#[tracing::instrument(skip_all)]
#[command(slash, context = "guild", description = "this is a test command")]
pub async fn test(_context: Context, _interaction: Interaction) -> Result<CommandResponse> {
	Ok(CommandResponse::Message { flags: None, content: Some("test, success!".into()), components: None })
}

#[tracing::instrument(skip_all)]
#[command(slash, context = "bot_dm", description = "this is a test command")]
pub async fn dm_test(_context: Context, _interaction: Interaction) -> Result<CommandResponse> {
	Ok(CommandResponse::Message { flags: None, content: Some("test, success!".into()), components: None })
}