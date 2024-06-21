use nikomail_cache::CACHE;
use nikomail_util::{ DISCORD_APP_ID, DISCORD_CLIENT };
use twilight_http::request::channel::reaction::RequestReactionType;
use twilight_model::{
	channel::message::ReactionType,
	gateway::payload::incoming::ReactionAdd
};

use crate::Result;

pub async fn reaction_add(reaction_add: ReactionAdd) -> Result<()> {
	if
		reaction_add.user_id.get() != DISCORD_APP_ID.get() &&
		let Some(relayed_message_ref) = CACHE.nikomail.relayed_message_by_ref(reaction_add.message_id).await? &&
		let Some(relayed_message) = relayed_message_ref.value() &&
		!relayed_message.is_topic_starter
	{
		let (channel_id, message_id) = relayed_message.message_other_ids(reaction_add.message_id);
		let reaction = match &reaction_add.emoji {
			ReactionType::Custom { animated: _, id, name } =>
				RequestReactionType::Custom { id: *id, name: name.as_ref().map(|x| x.as_str()) },
			ReactionType::Unicode { name } =>
				RequestReactionType::Unicode { name }
		};

		DISCORD_CLIENT.create_reaction(channel_id, message_id, &reaction)
			.await?;
	}

	Ok(())
}