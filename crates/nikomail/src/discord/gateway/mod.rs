use twilight_model::gateway::{
	payload::outgoing::update_presence::UpdatePresencePayload,
	presence::{ Status, Activity, ActivityType }
};
use twilight_gateway::{ Shard, Intents, ShardId, StreamExt, ConfigBuilder, MessageSender, EventTypeFlags };

pub mod event;

pub fn initialise() -> MessageSender {
	tracing::info!("initialising discord gateway");

	let config = ConfigBuilder::new(
		env!("DISCORD_BOT_TOKEN").to_string(),
			Intents::DIRECT_MESSAGES |
			Intents::DIRECT_MESSAGE_REACTIONS |
			Intents::DIRECT_MESSAGE_TYPING |
			Intents::GUILDS |
			Intents::GUILD_MESSAGES |
			Intents::GUILD_MESSAGE_REACTIONS |
			Intents::GUILD_MESSAGE_TYPING |
			Intents::MESSAGE_CONTENT
	)
		.presence(UpdatePresencePayload::new(vec![Activity {
			id: None,
			url: None,
			name: "burgers".into(),
			kind: ActivityType::Custom,
			emoji: None,
			flags: None,
			party: None,
			state: Some(std::env::var("DISCORD_STATUS_TEXT").unwrap_or("let's get topical!!".into())),
			assets: None,
			buttons: vec![],
			details: None,
			secrets: None,
			instance: None,
			created_at: None,
			timestamps: None,
			application_id: None
		}], false, None, Status::Online).unwrap())
		.build();
	let mut shard = Shard::with_config(ShardId::ONE, config);
	let message_sender = shard.sender();
	tokio::spawn(async move {
		while let Some(item) = shard.next_event(EventTypeFlags::all()).await {
			let Ok(event) = item else {
				tracing::warn!(source = ?item.unwrap_err(), "error receiving event");
				continue;
			};
	
			event::handle_event(event);
		}
	});

	message_sender
}