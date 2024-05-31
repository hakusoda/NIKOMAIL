use nikomail_util::PG_POOL;
use twilight_model::id::{
	marker::{ GuildMarker, ChannelMarker },
	Id
};

use crate::Result;

pub struct ServerModel {
	pub id: Id<GuildMarker>,
	pub forum_channel_id: Option<Id<ChannelMarker>>
}

impl ServerModel {
	pub async fn get(guild_id: Id<GuildMarker>) -> Result<Option<Self>> {
		Ok(sqlx::query!(
			"
			SELECT id, forum_channel_id
			FROM servers
			WHERE id = $1
			",
			guild_id.get() as i64
		)
			.fetch_optional(PG_POOL.get().unwrap())
			.await?
			.map(|record| Self {
				id: Id::new(record.id as u64),
				forum_channel_id: record.forum_channel_id.map(|x| Id::new(x as u64))
			})
		)
	}
}

impl From<Id<GuildMarker>> for ServerModel {
	fn from(value: Id<GuildMarker>) -> Self {
		Self {
			id: value,
			forum_channel_id: None
		}
	}
}