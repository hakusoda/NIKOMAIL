use dashmap::{
	mapref::one::{ Ref, RefMut },
	DashMap
};
use nikomail_models::nikomail::{ RelayedMessageModel, TopicModel, ServerModel, UserStateModel };
use std::collections::HashSet;
use twilight_model::id::{
	marker::{ ChannelMarker, GuildMarker, MessageMarker, UserMarker },
	Id
};	

use crate::Result;

#[derive(Default)]
pub struct NikomailCache {
	pub relayed_messages: DashMap<u64, Option<RelayedMessageModel>>,
	pub relayed_message_refs: DashMap<Id<MessageMarker>, u64>,
	pub topics: DashMap<Id<ChannelMarker>, Option<TopicModel>>,
	pub servers: DashMap<Id<GuildMarker>, ServerModel>,
	pub user_states: DashMap<Id<UserMarker>, UserStateModel>,
	pub user_topics: DashMap<Id<UserMarker>, HashSet<Id<ChannelMarker>>>
}

impl NikomailCache {
	pub async fn relayed_message(&self, id: u64) -> Result<Ref<'_, u64, Option<RelayedMessageModel>>> {
		Ok(match self.relayed_messages.get(&id) {
			Some(model) => model,
			None => {
				let new_model = RelayedMessageModel::get(id)
					.await?;
				self.relayed_messages
					.entry(id)
					.insert(new_model)
					.downgrade()
			}
		})
	}

	pub async fn relayed_message_by_ref(&self, message_id: Id<MessageMarker>) -> Result<Option<Ref<'_, u64, Option<RelayedMessageModel>>>> {
		Ok(if let Some(id) = self.relayed_message_refs.get(&message_id) {
			Some(self.relayed_message(*id).await?)
		} else { None })
	}

	pub async fn topic(&self, thread_id: Id<ChannelMarker>) -> Result<Ref<'_, Id<ChannelMarker>, Option<TopicModel>>> {
		Ok(match self.topics.get(&thread_id) {
			Some(model) => model,
			None => self
				._insert_topic(thread_id)
				.await?
				.downgrade()
		})
	}

	pub async fn topic_mut(&self, thread_id: Id<ChannelMarker>) -> Result<RefMut<Id<ChannelMarker>, Option<TopicModel>>> {
		Ok(match self.topics.get_mut(&thread_id) {
			Some(model) => model,
			None => self
				._insert_topic(thread_id)
				.await?
		})
	}

	async fn _insert_topic(&self, thread_id: Id<ChannelMarker>) -> Result<RefMut<Id<ChannelMarker>, Option<TopicModel>>> {
		let new_model = TopicModel::get(thread_id)
			.await?;
		Ok(self.topics
			.entry(thread_id)
			.insert(new_model)
		)
	}

	pub async fn server(&self, guild_id: Id<GuildMarker>) -> Result<Ref<'_, Id<GuildMarker>, ServerModel>> {
		Ok(match self.servers.get(&guild_id) {
			Some(model) => model,
			None => self
				._insert_server(guild_id)
				.await?
				.downgrade()
		})
	}

	pub async fn server_mut(&self, guild_id: Id<GuildMarker>) -> Result<RefMut<Id<GuildMarker>, ServerModel>> {
		Ok(match self.servers.get_mut(&guild_id) {
			Some(model) => model,
			None => self
				._insert_server(guild_id)
				.await?
		})
	}

	async fn _insert_server(&self, guild_id: Id<GuildMarker>) -> Result<RefMut<Id<GuildMarker>, ServerModel>> {
		let new_model = ServerModel::get(guild_id)
			.await?
			.unwrap_or_else(|| ServerModel::from(guild_id));
		Ok(self.servers
			.entry(guild_id)
			.insert(new_model)
		)
	}

	pub async fn user_state(&self, user_id: Id<UserMarker>) -> Result<Ref<'_, Id<UserMarker>, UserStateModel>> {
		Ok(match self.user_states.get(&user_id) {
			Some(model) => model,
			None => self
				._insert_user_state(user_id).
				await?
				.downgrade()
		})
	}

	pub async fn user_state_mut(&self, user_id: Id<UserMarker>) -> Result<RefMut<Id<UserMarker>, UserStateModel>> {
		Ok(match self.user_states.get_mut(&user_id) {
			Some(model) => model,
			None => self
				._insert_user_state(user_id)
				.await?
		})
	}

	async fn _insert_user_state(&self, user_id: Id<UserMarker>) -> Result<RefMut<Id<UserMarker>, UserStateModel>> {
		let new_model = UserStateModel::get(user_id)
			.await?
			.unwrap_or_else(|| UserStateModel::new(user_id));
		Ok(self.user_states
			.entry(user_id)
			.insert(new_model)
		)
	}

	pub async fn user_topics(&self, user_id: Id<UserMarker>) -> Result<Ref<'_, Id<UserMarker>, HashSet<Id<ChannelMarker>>>> {
		Ok(match self.user_topics.get(&user_id) {
			Some(model) => model,
			None => {
				let new_model_ids = TopicModel::get_many_user(user_id)
					.await?;
				let mut model = self.user_topics
					.entry(user_id)
					.or_default();
				model.extend(new_model_ids.into_iter());

				model.downgrade()
			}
		})
	}

	pub fn add_relayed_message(&self, relayed_message: RelayedMessageModel) {
		let id = relayed_message.id;
		self.relayed_message_refs.insert(relayed_message.source_message_id, id);
		self.relayed_message_refs.insert(relayed_message.relayed_message_id, id);
		self.relayed_messages.insert(id, Some(relayed_message));
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