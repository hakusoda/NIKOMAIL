use dashmap::{ mapref::one::{ Ref, RefMut }, DashMap };
use once_cell::sync::OnceCell;
use twilight_model::id::{
	marker::{ UserMarker, ChannelMarker, MessageMarker },
	Id
};

#[derive(Debug, Default)]
pub struct State {
	pub user_states: DashMap<Id<UserMarker>, UserState>,
	pub copied_message_sources: DashMap<(Id<ChannelMarker>, Id<MessageMarker>), (Id<ChannelMarker>, Id<MessageMarker>, bool)>
}

impl State {
	pub fn user_state(&self, user_id: Id<UserMarker>) -> Ref<'_, Id<UserMarker>, UserState> {
		match self.user_states.get(&user_id) {
			Some(model) => model,
			None => self.user_states.entry(user_id)
				.insert(UserState::default())
				.downgrade()
		}
	}

	pub fn user_state_mut(&self, user_id: Id<UserMarker>) -> RefMut<Id<UserMarker>, UserState> {
		match self.user_states.get_mut(&user_id) {
			Some(model) => model,
			None => self.user_states.entry(user_id)
				.insert(UserState::default())
		}
	}

	pub fn copied_message_source(&self, channel_id: Id<ChannelMarker>, message_id: Id<MessageMarker>) -> Option<Ref<'_, (Id<ChannelMarker>, Id<MessageMarker>), (Id<ChannelMarker>, Id<MessageMarker>, bool)>> {
		self.copied_message_sources.get(&(channel_id, message_id))
	}
}

#[derive(Debug, Default)]
pub struct UserState {
	pub current_topic_id: Option<Id<ChannelMarker>>
}

pub static STATE: OnceCell<State> = OnceCell::new();