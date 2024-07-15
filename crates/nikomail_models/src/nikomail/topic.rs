use futures::TryStreamExt;
use nikomail_util::PG_POOL;
use std::pin::Pin;
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
		Self::get_many(&[channel_id])
			.await
			.map(|x| x.into_iter().next())
	}

	pub async fn get_many(channel_ids: &[Id<ChannelMarker>]) -> Result<Vec<Self>> {
		let channel_ids: Vec<i64> = channel_ids
			.iter()
			.map(|x| x.get() as i64)
			.collect();
		Ok(sqlx::query!(
			"
			SELECT id, author_id, server_id
			FROM topics
			WHERE id = ANY($1)
			",
			&channel_ids
		)
			.fetch(&*Pin::static_ref(&PG_POOL).await)
			.try_fold(Vec::new(), |mut acc, record| {
				acc.push(Self {
					id: Id::new(record.id as u64),
					author_id: Id::new(record.author_id as u64),
					server_id: Id::new(record.server_id as u64)
				});

				async move { Ok(acc) }
			})
			.await?
		)
	}

	pub async fn get_all() -> Result<Vec<Self>> {
		Ok(sqlx::query!(
			"
			SELECT id, author_id, server_id
			FROM topics
			"
		)
			.fetch(&*Pin::static_ref(&PG_POOL).await)
			.try_fold(Vec::new(), |mut acc, record| {
				acc.push(Self {
					id: Id::new(record.id as u64),
					author_id: Id::new(record.author_id as u64),
					server_id: Id::new(record.server_id as u64)
				});

				async move { Ok(acc) }
			})
			.await?
		)
	}

	pub async fn get_many_user(user_id: Id<UserMarker>) -> Result<Vec<Id<ChannelMarker>>> {
		Ok(sqlx::query!(
			"
			SELECT id
			FROM topics
			WHERE author_id = $1
			",
			user_id.get() as i64
		)
			.fetch(&*Pin::static_ref(&PG_POOL).await)
			.try_fold(Vec::new(), |mut acc, record| {
				acc.push(Id::new(record.id as u64));

				async move { Ok(acc) }
			})
			.await?
		)
	}
}