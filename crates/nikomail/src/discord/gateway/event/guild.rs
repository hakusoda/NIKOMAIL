use nikomail_cache::CACHE;
use twilight_model::gateway::payload::incoming::{ GuildCreate, GuildUpdate, GuildDelete };

use crate::Result;

pub fn guild_create(guild_create: GuildCreate) -> Result<()> {
	if let GuildCreate::Available(guild) = guild_create {
		CACHE.discord.guilds.insert(guild.id, guild.into());
	}
	
	Ok(())
}

pub fn guild_update(guild_update: GuildUpdate) -> Result<()> {
	if let Some(mut guild) = CACHE.discord.guilds.get_mut(&guild_update.id) {
		guild.update(&guild_update);
	}

	Ok(())
}

pub fn guild_delete(guild_delete: GuildDelete) -> Result<()> {
	CACHE.discord.guilds.remove(&guild_delete.id);
	Ok(())
}