use twilight_model::{
	id::{
		marker::{ GuildMarker, ChannelMarker },
		Id
	},
	channel::{ Channel, ChannelType },
	gateway::payload::incoming::ChannelUpdate
};

#[derive(Eq, Clone, Debug, PartialEq)]
pub struct ChannelModel {
    pub guild_id: Option<Id<GuildMarker>>,
	pub id: Id<ChannelMarker>,
	pub kind: ChannelType,
	pub name: Option<String>
}

impl ChannelModel {
	pub fn update(&mut self, channel_update: &ChannelUpdate) {
		self.name.clone_from(&channel_update.name);
	}
}

impl std::hash::Hash for ChannelModel {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.id.hash(state);
	}
}

impl From<Channel> for ChannelModel {
	fn from(value: Channel) -> Self {
		let Channel {
			/*application_id,
			applied_tags,
			available_tags,
			bitrate,
			default_auto_archive_duration,
			default_forum_layout,
			default_reaction_emoji,
			default_sort_order,
			default_thread_rate_limit_per_user,
			flags,*/
			guild_id,
			//icon,
			id,
			//invitable,
			kind,
			/*last_message_id,
			last_pin_timestamp,
			managed,
			member,
			member_count,
			message_count,*/
			name,
			/*newly_created,
			nsfw,
			owner_id,
			parent_id,
			permission_overwrites,
			position,
			rate_limit_per_user,
			recipients,
			rtc_region,
			thread_metadata,
			topic,
			user_limit,
			video_quality_mode,*/
			..
		} = value;
		Self {
			guild_id,
			id,
			kind,
			name
		}
	}
}