use nikomail_util::{ PG_POOL, DISCORD_CLIENT };
use twilight_model::{
	id::Id,
	channel::message::{
		component::{ Button, ActionRow, ButtonStyle, Component },
		MessageFlags, ReactionType
	}
};
use nikomail_macros::command;

use crate::{
	CACHE,
	Result, Context, Interaction, CommandResponse
};

#[tracing::instrument(skip_all)]
#[command(slash, context = "guild", description = "Create a prewritten message with a topic creation button.", default_member_permissions = "8192")]
pub async fn create_button(_context: Context, _interaction: Interaction) -> Result<CommandResponse> {
	Ok(CommandResponse::Message {
		flags: None,
		content: Some("## <:niko_smile:1226793977232097321>  NIKOMAIL (working title)\nThis server uses an anonymous mailing system, for one-on-one conversations with server staff, without revealing anyone's identities.\n\n### ❓  How does it work?\nWhen a user creates a topic, they will be redirected to directly message me, where I will act as an anonymous relay between you and server staff.\nAttachments and links are permitted, along with emojis **in this server**, and default stickers **provided by Discord**.\n\n<:personbadge:1219233857786875925> *Keeping your identity hidden is **your** responsibility, try avoiding use of personal CDNs and the like.*\n‼️ *Duly note that staff are able to (still-anonymously) blacklist you from using NIKOMAIL when deemed necessary.*\n\nWith all that out of the way, simply tap the button below to start mailing server staff!".into()),
		components: Some(vec![
			Component::ActionRow(ActionRow {
				components: vec![
					Component::Button(Button {
						url: None,
						label: Some("Start new topic".into()),
						emoji: Some(ReactionType::Custom { animated: false, id: Id::new(1219234152709095424), name: Some("dap_me_up".into()) }),
						style: ButtonStyle::Primary,
						disabled: false,
						custom_id: Some("create_topic".into())
					})
				]
			})
		])
	})
}

#[tracing::instrument(skip_all)]
#[command(slash, context = "guild", description = "Blacklist the current topic author from using NIKOMAIL.", default_member_permissions = "17179869184")]
pub async fn blacklist_topic_author(_context: Context, interaction: Interaction) -> Result<CommandResponse> {
	Ok(CommandResponse::Message {
		flags: Some(MessageFlags::EPHEMERAL),
		content: Some({
			if let Some(current_topic) = CACHE.nikomail.topic(interaction.channel.unwrap().id).await?.value() {
				let guild_id = interaction.guild_id.unwrap();
				let mut server = CACHE.nikomail.server_mut(guild_id).await?;
				server.blacklisted_user_ids.push(current_topic.author_id);
				
				let user_ids = server.blacklisted_user_ids
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
					.execute(PG_POOL.get().unwrap())
					.await?;

				let private_channel_id = CACHE.discord.private_channel(current_topic.author_id).await?;
				DISCORD_CLIENT.create_message(*private_channel_id)
					.content("## You have been blacklisted in [UNTRACKED]\nunfortunately, server staff have decided to blacklist you from using NIKOMAIL, you will no longer be able to create new topics.")?
					.await?;

				"success! the author of this topic has been blacklisted from using NIKOMAIL.\n*they will still be able to talk in this topic until you delete the thread, sorry i was crunched for time on this part...*"
			} else {
				"hmmmmmm, this isn't familiar to me, are you sure you're executing this in the right place?"
			}.into()
		}),
		components: None
	})
}