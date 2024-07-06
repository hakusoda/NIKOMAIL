use nikomail_cache::CACHE;
use nikomail_util::{ DISCORD_APP_ID, DISCORD_CLIENT };
use twilight_model::gateway::payload::incoming::TypingStart;

use crate::Result;

pub async fn typing_start(typing_start: TypingStart) -> Result<()> {
	if typing_start.user_id.get() != DISCORD_APP_ID.get() {
		if typing_start.guild_id.is_some() {
			if let Some(topic) = CACHE.nikomail.topic(typing_start.channel_id).await?.value() {
				let private_channel_id = CACHE.discord
					.private_channel(topic.author_id)
					.await?;
				DISCORD_CLIENT
					.create_typing_trigger(private_channel_id)
					.await?;
			}
		} else {
			let user_state = CACHE.nikomail.user_state(typing_start.user_id).await?;
			if let Some(current_topic_id) = user_state.current_topic_id {
				DISCORD_CLIENT
					.create_typing_trigger(current_topic_id)
					.await?;
			}
		}
	}

	Ok(())
}