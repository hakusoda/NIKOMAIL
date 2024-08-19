use nikomail_cache::CACHE;
use nikomail_commands::util::CloseTopicOperation;
use twilight_model::gateway::payload::incoming::{ ThreadCreate, ThreadUpdate, ThreadDelete };

use crate::Result;

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

	if thread_update.thread_metadata.as_ref().is_some_and(|x| x.locked) {
		CloseTopicOperation::Generic
			.execute(thread_id)
			.await?;
	}

	Ok(())
}

pub async fn thread_delete(thread_delete: ThreadDelete) -> Result<()> {
	let thread_id = thread_delete.id;
	let channel = CACHE.discord.channels.remove(&thread_id);
	CloseTopicOperation::Deleted(channel.and_then(|x| x.1.name))
		.execute(thread_id)
		.await?;

	Ok(())
}