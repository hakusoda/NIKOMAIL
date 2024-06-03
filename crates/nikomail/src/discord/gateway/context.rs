use tokio::time::{ Duration, sleep };
use nikomail_util::{ PG_POOL, DISCORD_APP_ID, DISCORD_CLIENT, DISCORD_INTERACTION_CLIENT };
use twilight_http::request::channel::reaction::RequestReactionType;
use twilight_util::builder::InteractionResponseDataBuilder;
use twilight_model::{
	id::{
		marker::{ ChannelMarker, StickerMarker },
		Id
	},
	http::{
		attachment::Attachment,
		interaction::{ InteractionResponse, InteractionResponseData, InteractionResponseType }
	},
	channel::{
		thread::AutoArchiveDuration,
		message::{
			component::{ ActionRow, TextInput, TextInputStyle },
			MessageFlags, ReactionType
		}
	},
	gateway::payload::incoming::MessageCreate,
	application::interaction::InteractionData
};
use nikomail_models::nikomail::TopicModel;
use twilight_gateway::{ Event, MessageSender };

use crate::{
	util::create_topic_button,
	state::STATE,
	discord::{ interactions::handle_interaction, app_command_id },
	Result,
	CACHE
};

pub struct Context;

impl Context {
	pub fn new(_message_sender: MessageSender) -> Self {
		Self {}
	}

	pub async fn handle_event(self: crate::Context, event: Event) -> Result<()> {
		tracing::info!("handle_event kind: {:?}", event.kind());
		match event {
			Event::InteractionCreate(event_data) => {
				if let Some(data) = &event_data.data {
					if
						let InteractionData::MessageComponent(component_data) = data &&
						let Some(author_id) = event_data.author_id() &&
						let Some(guild_id) = match event_data.guild_id {
							Some(x) => if component_data.custom_id == "create_topic" { Some(x) } else { None },
							None => if component_data.custom_id.starts_with("create_topic_") {
								component_data.custom_id[13..].parse::<u64>().ok().and_then(Id::new_checked)
							} else { None }
						}
					{
						let server = CACHE.nikomail.server(guild_id).await?;
						if server.forum_channel_id.is_some() {
							if server.blacklisted_user_ids.contains(&author_id) {
								DISCORD_INTERACTION_CLIENT.create_response(event_data.id, &event_data.token, &InteractionResponse {
									kind: InteractionResponseType::ChannelMessageWithSource,
									data: Some(InteractionResponseData {
										flags: Some(MessageFlags::EPHEMERAL),
										content: Some("i'm sorry, but you have been blacklisted from using NIKOMAIL in this server.".into()),
										..Default::default()
									})
								}).await?;
							} else if CACHE.nikomail.user_topics(author_id).await?.is_empty() {
								DISCORD_INTERACTION_CLIENT.create_response(
									event_data.id,
									&event_data.token,
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
								DISCORD_INTERACTION_CLIENT.create_response(event_data.id, &event_data.token, &InteractionResponse {
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
								event_data.id,
								&event_data.token,
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
						let InteractionData::ModalSubmit(modal_data) = data &&
						modal_data.custom_id == "create_topic_modal" &&
						let Some(guild_id) = event_data.guild_id &&
						let Some(author_id) = event_data.author_id()
					{
						let server = CACHE.nikomail.server(guild_id).await?;
						if server.blacklisted_user_ids.contains(&author_id) {
							DISCORD_INTERACTION_CLIENT.create_response(event_data.id, &event_data.token, &InteractionResponse {
								kind: InteractionResponseType::ChannelMessageWithSource,
								data: Some(InteractionResponseData {
									flags: Some(MessageFlags::EPHEMERAL),
									content: Some("i'm sorry, but you have been blacklisted from using NIKOMAIL in this server.".into()),
									..Default::default()
								})
							}).await?;
						} else if let Some(forum_channel_id) = server.forum_channel_id {
							let private_channel_id = CACHE.discord.private_channel(author_id).await?;
							
							let topic_name = modal_data.components[0].components[0].value.as_ref().unwrap();
							let topic_message = modal_data.components[1].components[0].value.as_ref().unwrap();
							let bytes = DISCORD_CLIENT.create_forum_thread(
								forum_channel_id,
								topic_name
							)
								.auto_archive_duration(AutoArchiveDuration::Week)
								.message()
								.content(topic_message)?
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

							CACHE.nikomail.topics.insert(thread_id, Some(TopicModel {
								id: thread_id,
								author_id,
								server_id: guild_id
							}));
							CACHE.nikomail.add_user_topic(author_id, thread_id);

							if let Ok(response) = DISCORD_CLIENT.create_message(*private_channel_id.value())
								.content(&format!("## Topic has been created\n**{topic_name}** has been created, server staff will get back to you shortly.\nMessages from staff will appear here in this DM, feel free to add anything to this topic below while you wait.\n\nSwitch topics with </set_topic:{}>, close topics with </close_topic:{}>", app_command_id("set_topic").await.unwrap(), app_command_id("close_topic").await.unwrap()))?
								.await
							{
								let message = response.model().await?;
								STATE.copied_message_sources.insert((new_thread.channel.id, new_thread.message.id), (message.channel_id, message.id, true));

								STATE.user_state_mut(author_id).current_topic_id.replace(new_thread.channel.id);
								DISCORD_INTERACTION_CLIENT.create_response(event_data.id, &event_data.token, &InteractionResponse {
									kind: InteractionResponseType::ChannelMessageWithSource,
									data: Some(InteractionResponseData {
										flags: Some(MessageFlags::EPHEMERAL),
										content: Some(format!("Topic has been created, refer to <#{}>", *private_channel_id)),
										..Default::default()
									})
								}).await?;
							} else {
								DISCORD_INTERACTION_CLIENT.create_response(event_data.id, &event_data.token, &InteractionResponse {
									kind: InteractionResponseType::ChannelMessageWithSource,
									data: Some(InteractionResponseData {
										flags: Some(MessageFlags::EPHEMERAL),
										content: Some(format!("Topic has been created, but I'm unable to directly message you, check your privacy settings, and then execute </set_topic:1245261841974820915> in <#{}>", *private_channel_id)),
										..Default::default()
									})
								}).await?;
							}

							return Ok(());
						}
					}
				}
				handle_interaction(self, event_data.0).await?;
			},
			Event::MessageCreate(event_data) => {
				if !event_data.author.bot {
					tokio::spawn(async move {
						if event_data.guild_id.is_some() {
							let channel = CACHE.discord.channel(event_data.channel_id).await?;
							if channel.kind.is_thread() && let Some(topic) = CACHE.nikomail.topic(channel.id).await?.value() {
								let private_channel_id = CACHE.discord.private_channel(topic.author_id).await?;
								copy_message_and_send(*event_data, *private_channel_id.value())
									.await?;
							}
						} else {
							let topic_id = match event_data.referenced_message.as_ref().map(|x| STATE.copied_message_source(x.channel_id, x.id).map(|x| x.0)) {
								Some(x) => x,
								None => STATE.user_state(event_data.author.id).current_topic_id
							};
							if let Some(topic_id) = topic_id {
								copy_message_and_send(*event_data, topic_id)
									.await?;
							} else {
								DISCORD_CLIENT.create_message(event_data.channel_id)
									.content("You must set the topic you'd like to respond to using </set_topic:1245261841974820915>")?
									.reply(event_data.id)
									.await?;
							}
						}
						Ok::<(), crate::error::Error>(())
					});
				}
			},
			Event::MessageUpdate(event_data) => {
				if let Some((proxy_message_channel_id, proxy_message_id)) = STATE.copied_message_sources
					.iter()
					.find_map(|x| if x.value().1 == event_data.id { Some(*x.key()) } else { None })
				{
					let builder = DISCORD_CLIENT.update_message(proxy_message_channel_id, proxy_message_id)
						.embeds(event_data.embeds.as_deref())?
						.content(event_data.content.as_deref())?;

					builder.await?;
				}
			},
			Event::MessageDelete(event_data) => {
				if let Some((proxy_message_channel_id, proxy_message_id)) = STATE.copied_message_sources
					.iter()
					.find_map(|x| if x.value().1 == event_data.id { Some(*x.key()) } else { None })
				{
					if event_data.guild_id.is_some() {
						DISCORD_CLIENT.delete_message(proxy_message_channel_id, proxy_message_id)
							.await?;
					} else {
						DISCORD_CLIENT.create_message(proxy_message_channel_id)
							.content("This message was deleted on the author's end.")?
							.reply(proxy_message_id)
							.await?;
					}
				}
			},
			Event::ThreadCreate(event_data) => {
				CACHE.discord.channels.insert(event_data.id, (*event_data).into());
			},
			Event::ThreadUpdate(event_data) => {
				let thread_id = event_data.id;
				let mut channel = CACHE.discord.channels.get_mut(&thread_id);
				if let Some(ref mut channel) = channel {
					channel.update_from_thread(&event_data);
				}

				let channel_name = channel.and_then(|x| x.name.clone());
				if event_data.thread_metadata.as_ref().is_some_and(|x| x.locked || x.archived) {
					if let Some(topic) = CACHE.nikomail.topic_mut(thread_id).await?.take() {
						let author_id = topic.author_id;
						CACHE.nikomail.remove_user_topic(author_id, thread_id);
						STATE.user_state_mut(author_id).current_topic_id = None;

						sqlx::query!(
							"
							DELETE from topics
							WHERE id = $1
							",
							thread_id.get() as i64
						)
							.execute(&*std::pin::Pin::static_ref(&PG_POOL).await)
							.await?;

						let private_channel_id = CACHE.discord.private_channel(author_id).await?;
						DISCORD_CLIENT.create_message(*private_channel_id)
							.content(&format!("## Topic has been closed\n**{}** has been closed by server staff, it cannot be reopened, feel free to open another one!", channel_name.unwrap_or("Unknown Topic".into())))?
							.components(&[create_topic_button(Some(topic.server_id))])?
							.await?;
					}
				}
			},
			Event::ThreadDelete(event_data) => {
				let thread_id = event_data.id;
				let channel = CACHE.discord.channels.remove(&thread_id);

				if let Some(topic) = CACHE.nikomail.topic_mut(thread_id).await?.take() {
					let author_id = topic.author_id;
					CACHE.nikomail.remove_user_topic(author_id, thread_id);
					STATE.user_state_mut(author_id).current_topic_id = None;

					sqlx::query!(
						"
						DELETE from topics
						WHERE id = $1
						",
						thread_id.get() as i64
					)
						.execute(&*std::pin::Pin::static_ref(&PG_POOL).await)
						.await?;

					let private_channel_id = CACHE.discord.private_channel(author_id).await?;
					DISCORD_CLIENT.create_message(*private_channel_id)
						.content(&format!("## Topic has been closed\n**{}** has been closed & deleted by server staff, feel free to open another one!", channel.and_then(|x| x.1.name).unwrap_or("Unknown Topic".into())))?
						.components(&[create_topic_button(Some(topic.server_id))])?
						.await?;
				}
			},
			Event::ReactionAdd(event_data) => {
				if event_data.user_id.get() != DISCORD_APP_ID.get() {
					if let Some(copied_message_source) = STATE.copied_message_source(event_data.channel_id, event_data.message_id) {
						let (copied_message_channel_id, copied_message_id, is_thread_starter) = *copied_message_source;
						if !is_thread_starter {
							let reaction = match &event_data.emoji {
								ReactionType::Custom { animated: _, id, name } =>
									RequestReactionType::Custom { id: *id, name: name.as_ref().map(|x| x.as_str()) },
								ReactionType::Unicode { name } =>
									RequestReactionType::Unicode { name }
							};

							DISCORD_CLIENT.create_reaction(copied_message_channel_id, copied_message_id, &reaction)
								.await?;
						}
					}
				}
			},
			Event::TypingStart(event_data) => {
				if event_data.user_id.get() != DISCORD_APP_ID.get() {
					if event_data.guild_id.is_some() {
						if let Some(topic) = CACHE.nikomail.topic(event_data.channel_id).await?.value() {
							let private_channel_id = CACHE.discord.private_channel(topic.author_id).await?;
							DISCORD_CLIENT.create_typing_trigger(*private_channel_id)
								.await?;
						}
					} else {
						let user_state = STATE.user_state(event_data.user_id);
						if let Some(current_topic_id) = user_state.current_topic_id {
							DISCORD_CLIENT.create_typing_trigger(current_topic_id)
								.await?;
						}
					}
				}
			},
			_ => ()
		};
		Ok(())
	}
}

pub async fn copy_message_and_send(message: MessageCreate, channel_id: Id<ChannelMarker>) -> Result<()> {
	let result: Result<()> = try {
		let has_attachments = !message.attachments.is_empty();
		if has_attachments {
			tokio::spawn(DISCORD_CLIENT.create_reaction(message.channel_id, message.id, &RequestReactionType::Unicode { name: "⏳" }).into_future());
		}

		let mut attachments: Vec<Attachment> = vec![];
		for (index, attachment) in message.attachments.iter().enumerate() {
			let bytes = reqwest::get(&attachment.url)
				.await?
				.bytes()
				.await?;
			attachments.push(Attachment::from_bytes(attachment.filename.clone(), bytes.to_vec(), index as u64));
		}

		let sticker_ids = message.sticker_items.iter().map(|x| x.id).collect::<Vec<Id<StickerMarker>>>();
		let mut builder = DISCORD_CLIENT.create_message(channel_id)
			.content(&message.content)?
			.attachments(&attachments)?
			.sticker_ids(sticker_ids.as_slice())?;
		if
			let Some(referenced_message) = &message.referenced_message &&
			let Some(copied_message_source) = STATE.copied_message_source(referenced_message.channel_id, referenced_message.id)
		{
			builder = builder.reply(copied_message_source.1);
		}

		let new_message = builder
			.await?
			.model()
			.await?;

		STATE.copied_message_sources.insert((channel_id, new_message.id), (message.channel_id, message.id, false));
		if has_attachments {
			DISCORD_CLIENT.delete_current_user_reaction(message.channel_id, message.id, &RequestReactionType::Unicode { name: "⏳" })
				.await?;
			DISCORD_CLIENT.create_reaction(message.channel_id, message.id, &RequestReactionType::Unicode { name: "✅" })
				.await?;

			let message_id = message.id;
			let message_channel_id = message.channel_id;
			tokio::spawn(async move {
				sleep(Duration::from_secs(5)).await;
				DISCORD_CLIENT.delete_current_user_reaction(message_channel_id, message_id, &RequestReactionType::Unicode { name: "✅" })
					.await
					.unwrap();
			});
		}
	};
	match result {
		Ok(_) => Ok(()),
		Err(error) => {
			println!("{error}");
			DISCORD_CLIENT.create_message(message.channel_id)
				.content("oh dear, something went wrong while transmitting this message...")?
				.reply(message.id)
				.await?;
			Err(error)
		}
	}
}