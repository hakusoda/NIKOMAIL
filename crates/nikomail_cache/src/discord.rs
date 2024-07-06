use dashmap::{
	mapref::one::Ref,
	DashMap
};
use nikomail_util::DISCORD_CLIENT;
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
			None => self.channels.entry(channel_id)
				.insert(DISCORD_CLIENT.channel(channel_id).await?.model().await?.into())
				.downgrade()
		})
	}

	pub async fn guild(&self, guild_id: Id<GuildMarker>) -> Result<Ref<'_, Id<GuildMarker>, GuildModel>> {
		Ok(match self.guilds.get(&guild_id) {
			Some(model) => model,
			None => self.guilds.entry(guild_id)
				.insert(DISCORD_CLIENT.guild(guild_id).await?.model().await?.into())
				.downgrade()
		})
	}

	pub async fn private_channel(&self, user_id: Id<UserMarker>) -> Result<Id<ChannelMarker>> {
		Ok(*match self.private_channels.get(&user_id) {
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