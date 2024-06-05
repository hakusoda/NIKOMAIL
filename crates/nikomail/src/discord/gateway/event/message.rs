use tokio::time::{ Duration, sleep };
use nikomail_util::DISCORD_CLIENT;
use twilight_http::request::channel::reaction::RequestReactionType;
use twilight_model::{
	id::{
		marker::{ ChannelMarker, StickerMarker },
		Id
	},
	http::attachment::Attachment,
	gateway::payload::incoming::{ MessageCreate, MessageUpdate, MessageDelete }
};

use crate::{ state::STATE, Result, CACHE };

pub async fn message_create(message_create: MessageCreate) -> Result<()> {
	if !message_create.author.bot {
		if message_create.guild_id.is_some() {
			let channel = CACHE.discord.channel(message_create.channel_id).await?;
			if channel.kind.is_thread() && let Some(topic) = CACHE.nikomail.topic(channel.id).await?.value() {
				let private_channel_id = CACHE.discord.private_channel(topic.author_id).await?;
				copy_message_and_send(message_create, *private_channel_id.value())
					.await?;
			}
		} else {
			let topic_id = match message_create.referenced_message.as_ref().map(|x| STATE.copied_message_source(x.channel_id, x.id).map(|x| x.0)) {
				Some(x) => x,
				None => STATE.user_state(message_create.author.id).current_topic_id
			};
			if let Some(topic_id) = topic_id {
				copy_message_and_send(message_create, topic_id)
					.await?;
			} else {
				DISCORD_CLIENT.create_message(message_create.channel_id)
					.content("You must set the topic you'd like to respond to using </set_topic:1245261841974820915>")?
					.reply(message_create.id)
					.await?;
			}
		}
	}
	Ok(())
}

pub async fn message_update(message_update: MessageUpdate) -> Result<()> {
	if let Some((proxy_message_channel_id, proxy_message_id)) = STATE.copied_message_sources
		.iter()
		.find_map(|x| if x.value().1 == message_update.id { Some(*x.key()) } else { None })
	{
		let builder = DISCORD_CLIENT.update_message(proxy_message_channel_id, proxy_message_id)
			.embeds(message_update.embeds.as_deref())?
			.content(message_update.content.as_deref())?;

		builder.await?;
	}
	Ok(())
}

pub async fn message_delete(message_delete: MessageDelete) -> Result<()> {
	if let Some((proxy_message_channel_id, proxy_message_id)) = STATE.copied_message_sources
		.iter()
		.find_map(|x| if x.value().1 == message_delete.id { Some(*x.key()) } else { None })
	{
		if message_delete.guild_id.is_some() {
			DISCORD_CLIENT.delete_message(proxy_message_channel_id, proxy_message_id)
				.await?;
		} else {
			DISCORD_CLIENT.create_message(proxy_message_channel_id)
				.content("This message was deleted on the author's end.")?
				.reply(proxy_message_id)
				.await?;
		}
	}
	Ok(())
}

async fn copy_message_and_send(message: MessageCreate, channel_id: Id<ChannelMarker>) -> Result<()> {
	let result: Result<()> = try {
		let has_attachments = !message.attachments.is_empty();
		if has_attachments {
			tokio::spawn(DISCORD_CLIENT.create_reaction(message.channel_id, message.id, &RequestReactionType::Unicode { name: "⏳" }).into_future());
		}

		let mut attachments: Vec<Attachment> = vec![];
		for (index, attachment) in message.attachments.iter().enumerate() {
			let bytes = reqwest::get(&attachment.url)
				.await?
				.bytes()
				.await?;
			attachments.push(Attachment::from_bytes(attachment.filename.clone(), bytes.to_vec(), index as u64));
		}

		let sticker_ids = message.sticker_items.iter().map(|x| x.id).collect::<Vec<Id<StickerMarker>>>();
		let mut builder = DISCORD_CLIENT.create_message(channel_id)
			.content(&message.content)?
			.attachments(&attachments)?
			.sticker_ids(sticker_ids.as_slice())?;
		if
			let Some(referenced_message) = &message.referenced_message &&
			let Some(copied_message_source) = STATE.copied_message_source(referenced_message.channel_id, referenced_message.id)
		{
			builder = builder.reply(copied_message_source.1);
		}

		let new_message = builder
			.await?
			.model()
			.await?;

		STATE.copied_message_sources.insert((channel_id, new_message.id), (message.channel_id, message.id, false));
		if has_attachments {
			DISCORD_CLIENT.delete_current_user_reaction(message.channel_id, message.id, &RequestReactionType::Unicode { name: "⏳" })
				.await?;
			DISCORD_CLIENT.create_reaction(message.channel_id, message.id, &RequestReactionType::Unicode { name: "✅" })
				.await?;

			let message_id = message.id;
			let message_channel_id = message.channel_id;
			tokio::spawn(async move {
				sleep(Duration::from_secs(5)).await;
				DISCORD_CLIENT.delete_current_user_reaction(message_channel_id, message_id, &RequestReactionType::Unicode { name: "✅" })
					.await
					.unwrap();
			});
		}
	};
	match result {
		Ok(_) => Ok(()),
		Err(error) => {
			println!("{error}");
			DISCORD_CLIENT.create_message(message.channel_id)
				.content("oh dear, something went wrong while transmitting this message...")?
				.reply(message.id)
				.await?;
			Err(error)
		}
	}
}