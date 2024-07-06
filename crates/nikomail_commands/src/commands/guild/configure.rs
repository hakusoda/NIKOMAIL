use nikomail_cache::CACHE;
use nikomail_commands_core::{ Context, Result, command };
use nikomail_util::PG_POOL;
use twilight_model::id::{ marker::ChannelMarker, Id };

#[tracing::instrument(skip_all)]
#[command(slash, context = "guild", description = "Configure the settings for this server.", subcommands("forum_channel"), default_member_permissions = 32)]
pub async fn configure(_context: Context) -> Result<()> {
	unreachable!()
}

#[tracing::instrument(skip_all)]
#[command(slash, context = "guild", description = "Set the forum channel for topics to be created in.", default_member_permissions = 32)]
pub async fn forum_channel(
	context: Context,
	#[channel_kinds("guild_forum")]
	#[description = "the descripi"]
	new_channel: Id<ChannelMarker>
) -> Result<()> {
	let guild_id = context.guild_id().unwrap();
	sqlx::query!(
		"
		INSERT INTO servers (id, forum_channel_id)
		VALUES ($1, $2)
		ON CONFLICT (id)
		DO UPDATE SET forum_channel_id = $1
		",
		guild_id.get() as i64,
		new_channel.get() as i64
	)
		.execute(&*std::pin::Pin::static_ref(&PG_POOL).await)
		.await?;

	if let Some(mut server) = CACHE.nikomail.servers.get_mut(&guild_id) {
		server.forum_channel_id.replace(new_channel);
	}

	context.reply(format!("Successfully set the forum channel to <#{new_channel}>!"))
		.ephemeral()
		.await
}