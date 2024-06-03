use nikomail_util::PG_POOL;
use twilight_model::id::{
	marker::{ UserMarker, GuildMarker, ChannelMarker },
	Id
};

use crate::Result;

pub struct TopicModel {
	pub id: Id<ChannelMarker>,
	pub author_id: Id<UserMarker>,
	pub server_id: Id<GuildMarker>
}

impl TopicModel {
	pub async fn get(channel_id: Id<ChannelMarker>) -> Result<Option<Self>> {
		Ok(sqlx::query!(
			"
			SELECT id, author_id, server_id
			FROM topics
			WHERE id = $1
			",
			channel_id.get() as i64
		)
			.fetch_optional(&*std::pin::Pin::static_ref(&PG_POOL).await)
			.await?
			.map(|record| Self {
				id: Id::new(record.id as u64),
				author_id: Id::new(record.author_id as u64),
				server_id: Id::new(record.server_id as u64)
			})
		)
	}
}