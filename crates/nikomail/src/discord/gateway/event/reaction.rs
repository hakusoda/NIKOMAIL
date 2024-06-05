use nikomail_util::{ DISCORD_APP_ID, DISCORD_CLIENT };
use twilight_http::request::channel::reaction::RequestReactionType;
use twilight_model::{
	channel::message::ReactionType,
	gateway::payload::incoming::ReactionAdd
};

use crate::{ state::STATE, Result };

pub async fn reaction_add(reaction_add: ReactionAdd) -> Result<()> {
	if reaction_add.user_id.get() != DISCORD_APP_ID.get() {
		if let Some(copied_message_source) = STATE.copied_message_source(reaction_add.channel_id, reaction_add.message_id) {
			let (copied_message_channel_id, copied_message_id, is_thread_starter) = *copied_message_source;
			if !is_thread_starter {
				let reaction = match &reaction_add.emoji {
					ReactionType::Custom { animated: _, id, name } =>
						RequestReactionType::Custom { id: *id, name: name.as_ref().map(|x| x.as_str()) },
					ReactionType::Unicode { name } =>
						RequestReactionType::Unicode { name }
				};

				DISCORD_CLIENT.create_reaction(copied_message_channel_id, copied_message_id, &reaction)
					.await?;
			}
		}
	}

	Ok(())
}