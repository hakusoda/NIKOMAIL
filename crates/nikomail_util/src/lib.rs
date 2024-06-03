#![feature(const_async_blocks, type_alias_impl_trait)]
use sqlx::PgPool;
use once_cell::sync::Lazy;
use twilight_http::{ client::InteractionClient, Client };
use twilight_model::id::{ marker::ApplicationMarker, Id };
use async_once_cell::Lazy as AsyncLazy;

pub static DISCORD_CLIENT: Lazy<Client> = Lazy::new(|| Client::new(env!("DISCORD_BOT_TOKEN").to_owned()));
pub static DISCORD_INTERACTION_CLIENT: Lazy<InteractionClient> = Lazy::new(||
	DISCORD_CLIENT.interaction(*DISCORD_APP_ID)
);

pub static DISCORD_APP_ID: Lazy<Id<ApplicationMarker>> = Lazy::new(|| env!("DISCORD_APP_ID").to_owned().parse().unwrap());

pub type PgPoolFuture = impl Future<Output = PgPool>;
pub static PG_POOL: AsyncLazy<PgPool, PgPoolFuture> = AsyncLazy::new(async {
	PgPool::connect(env!("DATABASE_URL"))
		.await
		.unwrap()
});