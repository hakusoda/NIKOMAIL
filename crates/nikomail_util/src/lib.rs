use sqlx::PgPool;
use once_cell::sync::{ Lazy, OnceCell };
use twilight_http::{ client::InteractionClient, Client };
use twilight_model::id::{ marker::ApplicationMarker, Id };

pub static DISCORD_CLIENT: Lazy<Client> = Lazy::new(|| Client::new(env!("DISCORD_BOT_TOKEN").to_owned()));
pub static DISCORD_INTERACTION_CLIENT: Lazy<InteractionClient> = Lazy::new(||
	DISCORD_CLIENT.interaction(*DISCORD_APP_ID)
);

pub static DISCORD_APP_ID: Lazy<Id<ApplicationMarker>> = Lazy::new(|| env!("DISCORD_APP_ID").to_owned().parse().unwrap());

pub static PG_POOL: OnceCell<PgPool> = OnceCell::new();