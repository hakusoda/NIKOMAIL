use nikomail_util::PG_POOL;
use twilight_model::id::{
	marker::{ ChannelMarker, UserMarker },
	Id
};

use crate::Result;

pub struct UserStateModel {
	pub id: Id<UserMarker>,
	pub current_topic_id: Option<Id<ChannelMarker>>
}

impl UserStateModel {
	pub fn new(user_id: Id<UserMarker>) -> Self {
		Self {
			id: user_id,
			current_topic_id: None
		}
	}

	pub async fn get(user_id: Id<UserMarker>) -> Result<Option<Self>> {
		Ok(sqlx::query!(
			"
			SELECT id, current_topic_id
			FROM user_states
			WHERE id = $1
			",
			user_id.get() as i64
		)
			.fetch_optional(&*std::pin::Pin::static_ref(&PG_POOL).await)
			.await?
			.map(|record| Self {
				id: Id::new(record.id as u64),
				current_topic_id: record.current_topic_id.map(|x| Id::new(x as u64))
			})
		)
	}
}