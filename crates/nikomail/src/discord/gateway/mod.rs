use std::sync::{
	atomic::AtomicBool,
	Arc
};
use twilight_model::gateway::{
	payload::outgoing::update_presence::UpdatePresencePayload,
	presence::{ Status, Activity, ActivityType }
};
use twilight_gateway::{ Shard, Config, Intents, ShardId };

pub use context::Context;

pub mod event;
pub mod context;

pub async fn initialise() {
	tracing::info!("initialising discord gateway");

	let config = Config::builder(
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
	let context = Arc::new(Context::new(shard.sender()));
	
	let term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term)).unwrap();

	loop {
		/*if term.load(Ordering::Relaxed) {
			tracing::info!("SIGINT received, shutting down gateway...");
			shard.close(CloseFrame::NORMAL).await.unwrap();
			break;
		}*/

		let item = shard.next_event().await;
		let Ok(event) = item else {
			let source = item.unwrap_err();
			tracing::error!(?source, "error receiving event");

			if source.is_fatal() {
				break;
			}

			continue;
		};

		let context = Arc::clone(&context);
		tokio::spawn(async move {
			match context.handle_event(event).await {
				Ok(_) => (),
				Err(source) => tracing::warn!(?source, "error handling event")
			}
		});
	}
}