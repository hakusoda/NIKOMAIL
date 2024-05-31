use nikomail_util::{ PG_POOL, DISCORD_APP_ID, DISCORD_CLIENT, DISCORD_INTERACTION_CLIENT };
use twilight_http::request::channel::reaction::RequestReactionType;
use twilight_util::builder::InteractionResponseDataBuilder;
use twilight_model::{
	id::{
		marker::ChannelMarker,
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
	state::STATE,
	discord::interactions::handle_interaction,
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
						let Some(guild_id) = event_data.guild_id &&
						component_data.custom_id == "create_topic"
					{
						let server = CACHE.nikomail.server(guild_id).await?;
						if server.forum_channel_id.is_some() {
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
						if let Some(forum_channel_id) = server.forum_channel_id {
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
								.execute(PG_POOL.get().unwrap())
								.await?;

							CACHE.nikomail.topics.insert(thread_id, Some(TopicModel {
								id: thread_id,
								author_id,
								server_id: guild_id
							}));

							if let Ok(response) = DISCORD_CLIENT.create_message(*private_channel_id.value())
								.content(&format!("## Topic has been created\n**{topic_name}** has been created, server staff will get back to you shortly.\nMessages from staff will appear here in this DM, feel free to add anything to this topic below while you wait."))?
								.await
							{
								let state = STATE.get().unwrap();
								let message = response.model().await?;
								state.copied_message_sources.insert(new_thread.message.id, message.id);

								state.user_state(author_id).current_topic_id.replace(new_thread.channel.id);
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
					if event_data.guild_id.is_some() {
						let channel = CACHE.discord.channel(event_data.channel_id).await?;
						if channel.kind.is_thread() && let Some(topic) = CACHE.nikomail.topic(channel.id).await?.value() {
							let private_channel_id = CACHE.discord.private_channel(topic.author_id).await?;
							copy_message_and_send(*event_data, *private_channel_id.value())
								.await?;
						}
					} else {
						let user_state = STATE.get().unwrap().user_state(event_data.author.id);
						if let Some(current_topic_id) = user_state.current_topic_id {
							copy_message_and_send(*event_data, current_topic_id)
								.await?;
						} else {
							DISCORD_CLIENT.create_message(event_data.channel_id)
								.content("you gotta set a topic first using </set_topic:1245261841974820915>")?
								.reply(event_data.id)
								.await?;
						}
					}
				}
			},
			Event::ThreadDelete(event_data) => {
				if let Some((_,Some(topic))) = CACHE.nikomail.topics.remove(&event_data.id) {
					let author_id = topic.author_id;
					STATE.get().unwrap().user_state(author_id).current_topic_id = None;

					sqlx::query!(
						"
						DELETE from topics
						WHERE id = $1
						",
						event_data.id.get() as i64
					)
						.execute(PG_POOL.get().unwrap())
						.await?;

					let private_channel_id = CACHE.discord.private_channel(author_id).await?;
					DISCORD_CLIENT.create_message(*private_channel_id)
						.content("## Topic has been closed\n**[UNTRACKED]** has been closed & deleted by server staff.")?
						.await?;
				}
			},
			Event::ReactionAdd(event_data) => {
				if event_data.user_id.get() != DISCORD_APP_ID.get() {
					if let Some(copied_message_id) = STATE.get().unwrap().copied_message_source(event_data.message_id) {
						let reaction = match &event_data.emoji {
							ReactionType::Custom { animated: _, id, name } =>
								RequestReactionType::Custom { id: *id, name: name.as_ref().map(|x| x.as_str()) },
							ReactionType::Unicode { name } =>
								RequestReactionType::Unicode { name }
						};

						if event_data.guild_id.is_some() {
							if let Some(topic) = CACHE.nikomail.topic(event_data.channel_id).await?.value() {
								let private_channel_id = CACHE.discord.private_channel(topic.author_id).await?;
								DISCORD_CLIENT.create_reaction(*private_channel_id, *copied_message_id, &reaction)
									.await?;
							}
						} else {
							let user_state = STATE.get().unwrap().user_state(event_data.user_id);
							if let Some(current_topic_id) = user_state.current_topic_id {
								DISCORD_CLIENT.create_reaction(current_topic_id, *copied_message_id, &reaction)
									.await?;
							}
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
						let user_state = STATE.get().unwrap().user_state(event_data.user_id);
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
		let mut attachments: Vec<Attachment> = vec![];
		for (index, attachment) in message.attachments.iter().enumerate() {
			let bytes = reqwest::get(&attachment.url)
				.await?
				.bytes()
				.await?;
			attachments.push(Attachment::from_bytes(attachment.filename.clone(), bytes.to_vec(), index as u64));
		}

		let state = STATE.get().unwrap();
		let mut builder = DISCORD_CLIENT.create_message(channel_id)
			.content(&message.content)?
			.attachments(&attachments)?;
		if let Some(referenced_message) = &message.referenced_message && let Some(copied_message_id) = state.copied_message_source(referenced_message.id) {
			builder = builder.reply(*copied_message_id);
		}

		let new_message = builder
			.await?
			.model()
			.await?;

		state.copied_message_sources.insert(new_message.id, message.id);
	};
	match result {
		Ok(_) => Ok(()),
		Err(error) => {
			DISCORD_CLIENT.create_message(message.channel_id)
				.content("oh dear, something went wrong while transmitting this message...")?
				.reply(message.id)
				.await?;
			Err(error)
		}
	}
}