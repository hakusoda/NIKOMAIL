use nikomail_util::{ PG_POOL, DISCORD_CLIENT, DISCORD_INTERACTION_CLIENT };
use twilight_util::builder::InteractionResponseDataBuilder;
use twilight_model::{
	id::Id,
	http::interaction::{ InteractionResponse, InteractionResponseData, InteractionResponseType },
	channel::{
		thread::AutoArchiveDuration,
		message::{
			component::{ ActionRow, TextInput, TextInputStyle },
			MessageFlags
		}
	},
	gateway::payload::incoming::InteractionCreate,
	application::interaction::InteractionData
};
use nikomail_models::nikomail::{ RelayedMessageModel, TopicModel };

use crate::{
	discord::{ interactions::handle_interaction, app_command_id },
	Result,
	CACHE
};

pub async fn interaction_create(interaction_create: InteractionCreate) -> Result<()> {
	if let Some(data) = &interaction_create.data {
		if
			let InteractionData::MessageComponent(component_data) = data &&
			let Some(author_id) = interaction_create.author_id()
		{
			if let Some(guild_id) = match interaction_create.guild_id {
				Some(x) => if component_data.custom_id == "create_topic" { Some(x) } else { None },
				None => if component_data.custom_id.starts_with("create_topic_") {
					component_data.custom_id[13..].parse::<u64>().ok().and_then(Id::new_checked)
				} else { None }
			} {
				let server = CACHE
					.nikomail
					.server(guild_id)
					.await?;
				if server.forum_channel_id.is_some() {
					if server.blacklisted_user_ids.contains(&author_id) {
						DISCORD_INTERACTION_CLIENT.create_response(interaction_create.id, &interaction_create.token, &InteractionResponse {
							kind: InteractionResponseType::ChannelMessageWithSource,
							data: Some(InteractionResponseData {
								flags: Some(MessageFlags::EPHEMERAL),
								content: Some("i'm sorry, but you have been blacklisted from using NIKOMAIL in this server.".into()),
								..Default::default()
							})
						}).await?;
					} else if CACHE.nikomail.user_topics(author_id).await?.is_empty() {
						DISCORD_INTERACTION_CLIENT.create_response(
							interaction_create.id,
							&interaction_create.token,
							&InteractionResponse {
								kind: InteractionResponseType::Modal,
								data: Some(
									InteractionResponseDataBuilder::new()
										.title("Create a new topic")
										.custom_id("create_topic_modal")
										.components([
											ActionRow {
												components: vec![TextInput {
													value: None,
													label: "In short, what's your topic about?".into(),
													style: TextInputStyle::Short,
													required: Some(true),
													custom_id: "thread_name".into(),
													min_length: Some(1),
													max_length: Some(100),
													placeholder: Some("my concerns about the internal burgering...".into())
												}.into()]
											}.into(),
											ActionRow {
												components: vec![TextInput {
													value: None,
													label: "Starting message".into(),
													style: TextInputStyle::Paragraph,
													required: Some(true),
													custom_id: "initial_message".into(),
													min_length: Some(1),
													max_length: Some(2000),
													placeholder: Some("This should be your introduction to your topic, you can attach images and the like after this.".into())
												}.into()]
											}.into()
										])
										.build()
								)
							}
						).await?;
					} else {
						DISCORD_INTERACTION_CLIENT.create_response(interaction_create.id, &interaction_create.token, &InteractionResponse {
							kind: InteractionResponseType::ChannelMessageWithSource,
							data: Some(InteractionResponseData {
								flags: Some(MessageFlags::EPHEMERAL),
								content: Some("my apologies, we currently limit topics to one per user while we improve multi-topic interaction.".to_string()),
								..Default::default()
							})
						}).await?;
					}
				} else {
					DISCORD_INTERACTION_CLIENT.create_response(
						interaction_create.id,
						&interaction_create.token,
						&InteractionResponse {
							kind: InteractionResponseType::ChannelMessageWithSource,
							data: Some(
								InteractionResponseDataBuilder::new()
									.content("uhhhh no one set the forum channel which is pretty crazy, right? :(")
									.build()
							)
						}
					).await?;
				}
				return Ok(());
			} else if
				component_data.custom_id == "close_topic_menu" &&
				let Some(topic_id) = component_data.values
					.first()
					.and_then(|x| x.parse::<u64>().ok().and_then(Id::new_checked))
			{
				nikomail_commands::util::close_topic(interaction_create.id, &interaction_create.token, topic_id)
					.await?;
			}
		} else if 
			let InteractionData::ModalSubmit(modal_data) = data &&
			modal_data.custom_id == "create_topic_modal" &&
			let Some(guild_id) = interaction_create.guild_id &&
			let Some(author_id) = interaction_create.author_id()
		{
			let server = CACHE
				.nikomail
				.server(guild_id)
				.await?;
			if server.blacklisted_user_ids.contains(&author_id) {
				DISCORD_INTERACTION_CLIENT.create_response(interaction_create.id, &interaction_create.token, &InteractionResponse {
					kind: InteractionResponseType::ChannelMessageWithSource,
					data: Some(InteractionResponseData {
						flags: Some(MessageFlags::EPHEMERAL),
						content: Some("i'm sorry, but you have been blacklisted from using NIKOMAIL in this server.".into()),
						..Default::default()
					})
				}).await?;
			} else if let Some(forum_channel_id) = server.forum_channel_id {
				let private_channel_id = CACHE
					.discord
					.private_channel(author_id)
					.await?;
				
				let topic_name = modal_data.components[0].components[0].value.as_ref().unwrap();
				let topic_message = modal_data.components[1].components[0].value.as_ref().unwrap();
				let bytes = DISCORD_CLIENT.create_forum_thread(
					forum_channel_id,
					topic_name
				)
					.auto_archive_duration(AutoArchiveDuration::Week)
					.message()
					.content(topic_message)
					.await?
					.bytes()
					.await?;
				let new_thread: twilight_http::request::channel::thread::create_forum_thread::ForumThread = serde_json::from_slice(&bytes)?;
				
				let thread_id = new_thread.channel.id;
				sqlx::query!(
					"
					INSERT INTO topics
					VALUES ($1, $2, $3)
					",
					thread_id.get() as i64,
					author_id.get() as i64,
					guild_id.get() as i64
				)
					.execute(&*std::pin::Pin::static_ref(&PG_POOL).await)
					.await?;

				CACHE.nikomail.topics.insert(thread_id, TopicModel {
					id: thread_id,
					author_id,
					server_id: guild_id
				});
				CACHE.nikomail.add_user_topic(author_id, thread_id);
				CACHE.nikomail
					.user_state_mut(author_id)
					.await?
					.current_topic_id
					.replace(new_thread.channel.id);

				if let Ok(response) = DISCORD_CLIENT
					.create_message(private_channel_id)
					.content(&format!("## Topic has been created\n**{topic_name}** has been created, server staff will get back to you shortly.\nMessages from staff will appear here in this DM, feel free to add anything to this topic below while you wait.\n\nSwitch topics with </set_topic:{}>, close topics with </close_topic:{}>", app_command_id("set_topic").await.unwrap(), app_command_id("close_topic").await.unwrap()))
					.await
				{
					let message = response.model().await?;
					let relayed_message = RelayedMessageModel::insert(
						author_id,
						thread_id,
						thread_id,
						new_thread.message.id,
						message.channel_id,
						message.id,
						true
					).await?;
					CACHE.nikomail.add_relayed_message(relayed_message);
					
					DISCORD_INTERACTION_CLIENT
						.create_response(interaction_create.id, &interaction_create.token, &InteractionResponse {
							kind: InteractionResponseType::ChannelMessageWithSource,
							data: Some(InteractionResponseData {
								flags: Some(MessageFlags::EPHEMERAL),
								content: Some(format!("Topic has been created, refer to <#{}>", private_channel_id)),
								..Default::default()
							})
						})
						.await?;
				} else {
					DISCORD_INTERACTION_CLIENT
						.create_response(interaction_create.id, &interaction_create.token, &InteractionResponse {
							kind: InteractionResponseType::ChannelMessageWithSource,
							data: Some(InteractionResponseData {
								flags: Some(MessageFlags::EPHEMERAL),
								content: Some(format!("Topic has been created, but I'm unable to directly message you, check your privacy settings, and then execute </set_topic:1245261841974820915> in <#{}>", private_channel_id)),
								..Default::default()
							})
						})
						.await?;
				}

				return Ok(());
			}
		}
	}
	handle_interaction(interaction_create.0).await
}