use dashmap::{
	mapref::one::Ref,
	DashMap
};
use twilight_model::id::{
	marker::{ ChannelMarker, GuildMarker, UserMarker },
	Id
};	
use nikomail_models::discord::{ ChannelModel, GuildModel };

use crate::Result;

#[derive(Default)]
pub struct DiscordCache {
	pub channels: DashMap<Id<ChannelMarker>, ChannelModel>,
	pub guilds: DashMap<Id<GuildMarker>, GuildModel>,
	pub private_channels: DashMap<Id<UserMarker>, Id<ChannelMarker>>
}

impl DiscordCache {
	pub async fn channel(&self, channel_id: Id<ChannelMarker>) -> Result<Ref<'_, Id<ChannelMarker>, ChannelModel>> {
		Ok(match self.channels.get(&channel_id) {
			Some(model) => model,
			None => {
				let new_model = ChannelModel::get(channel_id)
					.await?;
				self.channels.entry(channel_id)
					.insert(new_model)
					.downgrade()
			}
		})
	}

	pub async fn guild(&self, guild_id: Id<GuildMarker>) -> Result<Ref<'_, Id<GuildMarker>, GuildModel>> {
		Ok(match self.guilds.get(&guild_id) {
			Some(model) => model,
			None => {
				let new_model = GuildModel::get(guild_id)
					.await?;
				self.guilds
					.entry(guild_id)
					.insert(new_model)
					.downgrade()
			}
		})
	}

	pub async fn private_channel(&self, user_id: Id<UserMarker>) -> Result<Id<ChannelMarker>> {
		Ok(*match self.private_channels.get(&user_id) {
			Some(model) => model,
			None => {
				let new_model = ChannelModel::get_private(user_id)
					.await?;
				let new_model_id = new_model.id;

				self.channels.insert(new_model_id, new_model);
				self.private_channels.entry(user_id)
					.insert(new_model_id)
					.downgrade()
			}
		})
	}
}