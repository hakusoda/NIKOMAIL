use nikomail_cache::CACHE;
use nikomail_util::{ PG_POOL, DISCORD_CLIENT };
use twilight_model::gateway::payload::incoming::{ ThreadCreate, ThreadUpdate, ThreadDelete };

use crate::{
	util::create_topic_button,
	Result
};

pub fn thread_create(thread_create: ThreadCreate) -> Result<()> {
	CACHE.discord.channels.insert(thread_create.id, thread_create.into());
	Ok(())
}

pub async fn thread_update(thread_update: ThreadUpdate) -> Result<()> {
	let thread_id = thread_update.id;
	let mut channel = CACHE.discord.channels.get_mut(&thread_id);
	if let Some(ref mut channel) = channel {
		channel.update_from_thread(&thread_update);
	}

	let channel_name = channel.and_then(|x| x.name.clone());
	if thread_update.thread_metadata.as_ref().is_some_and(|x| x.locked || x.archived) {
		if let Some(topic) = CACHE.nikomail.topic_mut(thread_id).await?.take() {
			let author_id = topic.author_id;
			let guild_id = topic.server_id;
			CACHE.nikomail.remove_user_topic(author_id, thread_id);
			CACHE.nikomail.user_state_mut(author_id).await?.current_topic_id = None;

			sqlx::query!(
				"
				DELETE from topics
				WHERE id = $1
				",
				thread_id.get() as i64
			)
				.execute(&*std::pin::Pin::static_ref(&PG_POOL).await)
				.await?;

			let private_channel_id = CACHE.discord
				.private_channel(author_id)
				.await?;
			DISCORD_CLIENT.create_message(private_channel_id)
				.content(&format!("## Topic has been closed\n**{}** has been closed by server staff, it cannot be reopened, feel free to open another one!", channel_name.unwrap_or("Unknown Topic".into())))
				.components(&[create_topic_button(Some(guild_id))])
				.await?;
		}
	}

	Ok(())
}

pub async fn thread_delete(thread_delete: ThreadDelete) -> Result<()> {
	let thread_id = thread_delete.id;
	let channel = CACHE.discord.channels.remove(&thread_id);
	if let Some(topic) = CACHE.nikomail.topic_mut(thread_id).await?.take() {
		let author_id = topic.author_id;
		let guild_id = topic.server_id;
		CACHE.nikomail.remove_user_topic(author_id, thread_id);
		CACHE.nikomail.user_state_mut(author_id).await?.current_topic_id = None;

		sqlx::query!(
			"
			DELETE from topics
			WHERE id = $1
			",
			thread_id.get() as i64
		)
			.execute(&*std::pin::Pin::static_ref(&PG_POOL).await)
			.await?;

		let private_channel_id = CACHE.discord
			.private_channel(author_id)
			.await?;
		DISCORD_CLIENT.create_message(private_channel_id)
			.content(&format!("## Topic has been closed\n**{}** has been closed & deleted by server staff, feel free to open another one!", channel.and_then(|x| x.1.name).unwrap_or("Unknown Topic".into())))
			.components(&[create_topic_button(Some(guild_id))])
			.await?;
	}

	Ok(())
}