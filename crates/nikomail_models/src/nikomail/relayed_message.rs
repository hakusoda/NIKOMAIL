use futures::TryStreamExt;
use nikomail_util::PG_POOL;
use twilight_model::id::{
	marker::{ ChannelMarker, MessageMarker, UserMarker },
	Id
};

use crate::Result;

pub struct RelayedMessageModel {
	pub id: u64,
	pub author_id: Id<UserMarker>,
	pub topic_id: Id<ChannelMarker>,
	pub source_channel_id: Id<ChannelMarker>,
	pub source_message_id: Id<MessageMarker>,
	pub relayed_channel_id: Id<ChannelMarker>,
	pub relayed_message_id: Id<MessageMarker>,
	pub is_topic_starter: bool
}

impl RelayedMessageModel {
	pub fn message_other_ids(&self, message_id: Id<MessageMarker>) -> (Id<ChannelMarker>, Id<MessageMarker>) {
		if message_id == self.source_message_id {
			(self.relayed_channel_id, self.relayed_message_id)
		} else { (self.source_channel_id, self.source_message_id) }
	}

	pub fn other_message_id(&self, id: Id<MessageMarker>) -> Id<MessageMarker> {
		if id == self.source_message_id {
			self.relayed_message_id
		} else { self.source_message_id }
	}

	pub async fn insert(
		author_id: Id<UserMarker>,
		topic_id: Id<ChannelMarker>,
		source_channel_id: Id<ChannelMarker>,
		source_message_id: Id<MessageMarker>,
		relayed_channel_id: Id<ChannelMarker>,
		relayed_message_id: Id<MessageMarker>,
		is_topic_starter: bool
	) -> Result<Self> {
		let record = sqlx::query!(
			"
			INSERT INTO relayed_messages (author_id, topic_id, source_channel_id, source_message_id, relayed_channel_id, relayed_message_id, is_topic_starter)
			VALUES ($1, $2, $3, $4, $5, $6, $7)
			RETURNING id
			",
			author_id.get() as i64,
			topic_id.get() as i64,
			source_channel_id.get() as i64,
			source_message_id.get() as i64,
			relayed_channel_id.get() as i64,
			relayed_message_id.get() as i64,
			is_topic_starter
		)
			.fetch_one(&*std::pin::Pin::static_ref(&PG_POOL).await)
			.await?;
		Ok(Self {
			id: record.id as u64,
			author_id,
			topic_id,
			source_channel_id,
			source_message_id,
			relayed_channel_id,
			relayed_message_id,
			is_topic_starter
		})
	}

	pub async fn get(id: u64) -> Result<Option<Self>> {
		Ok(sqlx::query!(
			"
			SELECT id, author_id, topic_id, source_channel_id, source_message_id, relayed_channel_id, relayed_message_id, is_topic_starter
			FROM relayed_messages
			WHERE id = $1
			",
			id as i64
		)
			.fetch_optional(&*std::pin::Pin::static_ref(&PG_POOL).await)
			.await?
			.map(|record| Self {
				id: record.id as u64,
				author_id: Id::new(record.author_id as u64),
				topic_id: Id::new(record.topic_id as u64),
				source_channel_id: Id::new(record.source_channel_id as u64),
				source_message_id: Id::new(record.source_message_id as u64),
				relayed_channel_id: Id::new(record.relayed_channel_id as u64),
				relayed_message_id: Id::new(record.relayed_message_id as u64),
				is_topic_starter: record.is_topic_starter
			})
		)
	}

	pub async fn get_all() -> Result<Vec<Self>> {
		Ok(sqlx::query!(
			"
			SELECT id, author_id, topic_id, source_channel_id, source_message_id, relayed_channel_id, relayed_message_id, is_topic_starter
			FROM relayed_messages
			"
		)
			.fetch(&*std::pin::Pin::static_ref(&PG_POOL).await)
			.try_fold(Vec::new(), |mut acc, record| {
				acc.push(Self {
					id: record.id as u64,
					author_id: Id::new(record.author_id as u64),
					topic_id: Id::new(record.topic_id as u64),
					source_channel_id: Id::new(record.source_channel_id as u64),
					source_message_id: Id::new(record.source_message_id as u64),
					relayed_channel_id: Id::new(record.relayed_channel_id as u64),
					relayed_message_id: Id::new(record.relayed_message_id as u64),
					is_topic_starter: record.is_topic_starter
				});
				async move { Ok(acc) }
			})
			.await?
		)
	}
}