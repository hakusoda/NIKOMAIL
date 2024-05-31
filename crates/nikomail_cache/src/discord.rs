use dashmap::{
	mapref::one::Ref,
	DashMap
};
use nikomail_util::DISCORD_CLIENT;
use twilight_model::id::{
	marker::{ UserMarker, ChannelMarker },
	Id
};	
use nikomail_models::discord::ChannelModel;

use crate::Result;

#[derive(Default)]
pub struct DiscordCache {
	pub channels: DashMap<Id<ChannelMarker>, ChannelModel>,
	pub private_channels: DashMap<Id<UserMarker>, Id<ChannelMarker>>
}

impl DiscordCache {
	pub async fn channel(&self, channel_id: Id<ChannelMarker>) -> Result<Ref<'_, Id<ChannelMarker>, ChannelModel>> {
		Ok(match self.channels.get(&channel_id) {
			Some(model) => model,
			None => self.channels.entry(channel_id)
				.insert(DISCORD_CLIENT.channel(channel_id).await?.model().await?.into())
				.downgrade()
		})
	}

	pub async fn private_channel(&self, user_id: Id<UserMarker>) -> Result<Ref<'_, Id<UserMarker>, Id<ChannelMarker>>> {
		Ok(match self.private_channels.get(&user_id) {
			Some(model) => model,
			None => self.private_channels.entry(user_id)
				.insert({
					let new_channel = DISCORD_CLIENT.create_private_channel(user_id).await?.model().await?;
					let new_channel_id = new_channel.id;
					self.channels.insert(new_channel_id, new_channel.into());
					
					new_channel_id
				})
				.downgrade()
		})
	}
}