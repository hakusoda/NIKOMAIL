use nikomail_cache::CACHE;
use nikomail_models::nikomail::RelayedMessageModel;
use nikomail_util::DISCORD_CLIENT;
use tokio::time::{ Duration, sleep };
use twilight_http::request::channel::reaction::RequestReactionType;
use twilight_model::{
	id::{
		marker::{ ChannelMarker, StickerMarker },
		Id
	},
	http::attachment::Attachment,
	gateway::payload::incoming::{ MessageCreate, MessageUpdate, MessageDelete }
};

use crate::Result;

pub async fn message_create(message_create: MessageCreate) -> Result<()> {
	if !message_create.author.bot {
		if message_create.guild_id.is_some() {
			let channel = CACHE.discord.channel(message_create.channel_id).await?;
			if channel.kind.is_thread() && let Some(topic) = CACHE.nikomail.topic(channel.id).await?.value() {
				let private_channel_id = CACHE.discord.private_channel(topic.author_id).await?;
				copy_message_and_send(message_create, private_channel_id, channel.id)
					.await?;
			}
		} else {
			let related_topic_id = match &message_create.referenced_message {
				Some(message) => match CACHE.nikomail.relayed_message_by_ref(message.id).await? {
					Some(x) => x.as_ref().map(|x| x.topic_id),
					None => None
				},
				None => None
			};
			let topic_id = match related_topic_id {
				Some(x) => Some(x),
				None => CACHE.nikomail.user_state(message_create.author.id).await?.current_topic_id
			};
			if let Some(topic_id) = topic_id {
				copy_message_and_send(message_create, topic_id, topic_id)
					.await?;
			} else {
				DISCORD_CLIENT.create_message(message_create.channel_id)
					.content("You must set the topic you'd like to respond to using </set_topic:1245261841974820915>")
					.reply(message_create.id)
					.await?;
			}
		}
	}
	Ok(())
}

pub async fn message_update(message_update: MessageUpdate) -> Result<()> {
	if
		let Some(relayed_message_ref) = CACHE.nikomail.relayed_message_by_ref(message_update.id).await? &&
		let Some(relayed_message) = relayed_message_ref.value()
	{
		let (channel_id, message_id) = relayed_message.message_other_ids(message_update.id);
		let builder = DISCORD_CLIENT.update_message(channel_id, message_id)
			.embeds(message_update.embeds.as_deref())
			.content(message_update.content.as_deref());

		builder.await?;
	}
	Ok(())
}

pub async fn message_delete(message_delete: MessageDelete) -> Result<()> {
	if
		let Some(relayed_message_ref) = CACHE.nikomail.relayed_message_by_ref(message_delete.id).await? &&
		let Some(relayed_message) = relayed_message_ref.value()
	{
		let (channel_id, message_id) = relayed_message.message_other_ids(message_delete.id);
		if message_delete.guild_id.is_some() {
			DISCORD_CLIENT.delete_message(channel_id, message_id)
				.await?;
		} else {
			DISCORD_CLIENT.create_message(channel_id)
				.content("This message was deleted on the author's end.")
				.reply(message_id)
				.await?;
		}
	}
	Ok(())
}

async fn copy_message_and_send(message: MessageCreate, channel_id: Id<ChannelMarker>, topic_id: Id<ChannelMarker>) -> Result<()> {
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
			.content(&message.content)
			.attachments(&attachments)
			.sticker_ids(sticker_ids.as_slice());
		if
			let Some(referenced_message) = &message.referenced_message &&
			let Some(relayed_message_ref) = CACHE.nikomail.relayed_message_by_ref(referenced_message.id).await? &&
			let Some(relayed_message) = relayed_message_ref.value()
		{
			builder = builder.reply(relayed_message.other_message_id(referenced_message.id));
		}

		let new_message = builder
			.await?
			.model()
			.await?;

		let relayed_message = RelayedMessageModel::insert(
			message.author.id,
			topic_id,
			message.channel_id,
			message.id,
			channel_id,
			new_message.id,
			false
		).await?;
		CACHE.nikomail.add_relayed_message(relayed_message);
		
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
				.content("oh dear, something went wrong while transmitting this message...")
				.reply(message.id)
				.await?;
			Err(error)
		}
	}
}