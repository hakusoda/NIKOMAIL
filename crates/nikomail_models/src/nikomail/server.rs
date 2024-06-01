use nikomail_util::PG_POOL;
use twilight_model::id::{
	marker::{ UserMarker, GuildMarker, ChannelMarker },
	Id
};

use crate::Result;

pub struct ServerModel {
	pub id: Id<GuildMarker>,
	pub blacklisted_user_ids: Vec<Id<UserMarker>>,
	pub forum_channel_id: Option<Id<ChannelMarker>>
}

impl ServerModel {
	pub async fn get(guild_id: Id<GuildMarker>) -> Result<Option<Self>> {
		Ok(sqlx::query!(
			"
			SELECT id, blacklisted_user_ids, forum_channel_id
			FROM servers
			WHERE id = $1
			",
			guild_id.get() as i64
		)
			.fetch_optional(PG_POOL.get().unwrap())
			.await?
			.map(|record| Self {
				id: Id::new(record.id as u64),
				blacklisted_user_ids: record.blacklisted_user_ids.into_iter().map(|x| Id::new(x as u64)).collect(),
				forum_channel_id: record.forum_channel_id.map(|x| Id::new(x as u64))
			})
		)
	}
}

impl From<Id<GuildMarker>> for ServerModel {
	fn from(value: Id<GuildMarker>) -> Self {
		Self {
			id: value,
			blacklisted_user_ids: vec![],
			forum_channel_id: None
		}
	}
}