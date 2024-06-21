use once_cell::sync::Lazy;

use discord::DiscordCache;
use nikomail::NikomailCache;

pub mod error;
pub mod discord;
pub mod nikomail;

pub use error::{ Error, Result };

#[derive(Default)]
pub struct Cache {
	pub discord: DiscordCache,
	pub nikomail: NikomailCache
}

pub static CACHE: Lazy<Cache> = Lazy::new(Cache::default);