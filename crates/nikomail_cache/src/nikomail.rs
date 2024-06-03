use std::collections::HashSet;
use dashmap::{
	mapref::one::{ Ref, RefMut },
	DashMap
};
use futures::stream::TryStreamExt;
use nikomail_util::PG_POOL;
use twilight_model::id::{
	marker::{ UserMarker, GuildMarker, ChannelMarker },
	Id
};	
use nikomail_models::nikomail::{ TopicModel, ServerModel };

use crate::Result;

#[derive(Default)]
pub struct NikomailCache {
	pub topics: DashMap<Id<ChannelMarker>, Option<TopicModel>>,
	pub servers: DashMap<Id<GuildMarker>, ServerModel>,
	pub user_topics: DashMap<Id<UserMarker>, HashSet<Id<ChannelMarker>>>
}

impl NikomailCache {
	pub async fn topic(&self, thread_id: Id<ChannelMarker>) -> Result<Ref<'_, Id<ChannelMarker>, Option<TopicModel>>> {
		self.topic_mut(thread_id).await.map(RefMut::downgrade)
	}

	pub async fn topic_mut(&self, thread_id: Id<ChannelMarker>) -> Result<RefMut<Id<ChannelMarker>, Option<TopicModel>>> {
		Ok(match self.topics.get_mut(&thread_id) {
			Some(model) => model,
			None => self.topics.entry(thread_id)
				.insert(TopicModel::get(thread_id).await?)
		})
	}

	pub async fn server(&self, guild_id: Id<GuildMarker>) -> Result<Ref<'_, Id<GuildMarker>, ServerModel>> {
		self.server_mut(guild_id).await.map(RefMut::downgrade)
	}

	pub async fn server_mut(&self, guild_id: Id<GuildMarker>) -> Result<RefMut<Id<GuildMarker>, ServerModel>> {
		Ok(match self.servers.get_mut(&guild_id) {
			Some(model) => model,
			None => self.servers.entry(guild_id)
				.insert(ServerModel::get(guild_id).await?.unwrap_or_else(|| ServerModel::from(guild_id)))
		})
	}

	pub async fn user_topics(&self, user_id: Id<UserMarker>) -> Result<Ref<'_, Id<UserMarker>, HashSet<Id<ChannelMarker>>>> {
		Ok(match self.user_topics.get(&user_id) {
			Some(model) => model,
			None => self.user_topics.entry(user_id)
				.insert(sqlx::query!(
					"
					SELECT id
					FROM topics
					WHERE author_id = $1
					",
					user_id.get() as i64
				)
					.fetch(&*std::pin::Pin::static_ref(&PG_POOL).await)
					.try_fold(HashSet::new(), |mut acc, m| {
						acc.insert(Id::new(m.id as u64));
						async move { Ok(acc) }
					})
					.await?
				)
				.downgrade()
		})
	}

	pub fn add_user_topic(&self, user_id: Id<UserMarker>, thread_id: Id<ChannelMarker>) {
		if let Some(mut user_topics) = self.user_topics.get_mut(&user_id) {
			user_topics.insert(thread_id);
		}
	}

	pub fn remove_user_topic(&self, user_id: Id<UserMarker>, thread_id: Id<ChannelMarker>) {
		if let Some(mut user_topics) = self.user_topics.get_mut(&user_id) {
			user_topics.remove(&thread_id);
		}
	}
}