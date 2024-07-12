use nikomail_cache::CACHE;
use nikomail_commands_core::{ Context, Result, command };
use nikomail_util::{ PG_POOL, DISCORD_CLIENT };
use std::pin::Pin;

use crate::util::create_topic_button;

mod configure;
pub use configure::configure;

#[tracing::instrument(skip_all)]
#[command(slash, context = "guild", description = "Create a prewritten message with a topic creation button.", default_member_permissions = 8192)]
pub async fn create_button(context: Context) -> Result<()> {
	context.reply("## <:niko_smile:1226793977232097321>  NIKOMAIL (working title)\nThis server uses an anonymous mailing system, for one-on-one conversations with server staff, without revealing anyone's identities.\n\n### ❓  How does it work?\nWhen a user creates a topic, they will be redirected to directly message me, where I will act as an anonymous relay between you and server staff.\nAttachments and links are permitted, along with emojis **in this server**, and default stickers **provided by Discord**.\n\n<:personbadge:1219233857786875925> *Keeping your identity hidden is **your** responsibility, try avoiding use of personal CDNs and the like.*\n‼️ *Duly note that staff are able to (still-anonymously) blacklist you from using NIKOMAIL when deemed necessary.*\n\nWith all that out of the way, simply tap the button below to start mailing server staff!")
		.components([create_topic_button(None)])
		.await
}

#[tracing::instrument(skip_all)]
#[command(slash, context = "guild", description = "Blacklist the current topic author from using NIKOMAIL.", default_member_permissions = 17179869184)]
pub async fn blacklist_topic_author(context: Context) -> Result<()> {
	context.reply({
		if let Some(current_topic) = CACHE.nikomail.topic(context.channel_id().unwrap()).await?.value() {
			let author_id = current_topic.author_id;
			let guild_id = context.guild_id().unwrap();
			let mut server = CACHE
				.nikomail
				.server_mut(guild_id)
				.await?;
			server.blacklisted_user_ids.push(author_id);
			
			let user_ids = server
				.blacklisted_user_ids
				.iter()
				.map(|x| x.get() as i64)
				.collect::<Vec<i64>>();
			sqlx::query!(
				"
				UPDATE servers
				SET blacklisted_user_ids = $1
				WHERE id = $2
				",
				user_ids.as_slice(),
				guild_id.get() as i64
			)
				.execute(&*Pin::static_ref(&PG_POOL).await)
				.await?;

			let private_channel_id = CACHE
				.discord
				.private_channel(author_id)
				.await?;
			let guild = CACHE
				.discord
				.guild(guild_id)
				.await?;
			DISCORD_CLIENT
				.create_message(private_channel_id)
				.content(&format!("## You have been blacklisted in {}\nUnfortunately, server staff have decided to blacklist you from using NIKOMAIL, you will no longer be able to create new topics.", guild.name))
				.await?;

			"success! the author of this topic has been blacklisted from using NIKOMAIL.\n*they will still be able to talk in this topic until you delete the thread, sorry i was crunched for time on this part...*"
		} else {
			"hmmmmmm, this isn't familiar to me, are you sure you're executing this in the right place?"
		}
	})
		.ephemeral()
		.await
}