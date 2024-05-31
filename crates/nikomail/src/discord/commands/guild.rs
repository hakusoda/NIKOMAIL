use twilight_model::{
	id::Id,
	channel::message::{
		component::{ Button, ActionRow, ButtonStyle, Component },
		ReactionType
	}
};
use nikomail_macros::command;

use crate::{
	Result, Context, Interaction, CommandResponse
};

#[tracing::instrument(skip_all)]
#[command(slash, context = "guild", description = "Create a prewritten message with a topic creation button.", default_member_permissions = "8192")]
pub async fn create_button(_context: Context, _interaction: Interaction) -> Result<CommandResponse> {
	Ok(CommandResponse::Message {
		flags: None,
		content: Some("Press the button below to start a private conversation with this server's staff, this will be fully anonymous.".into()),
		components: Some(vec![
			Component::ActionRow(ActionRow {
				components: vec![
					Component::Button(Button {
						url: None,
						label: Some("Start new topic".into()),
						emoji: Some(ReactionType::Custom { animated: false, id: Id::new(1219234152709095424), name: Some("dap_me_up".into()) }),
						style: ButtonStyle::Primary,
						disabled: false,
						custom_id: Some("create_topic".into())
					})
				]
			})
		])
	})
}