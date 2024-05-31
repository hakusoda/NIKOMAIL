pub mod error;
pub mod discord;
pub mod nikomail;

use discord::DiscordCache;
use nikomail::NikomailCache;

pub use error::{ Error, Result };

#[derive(Default)]
pub struct Cache {
	pub discord: DiscordCache,
	pub nikomail: NikomailCache
}