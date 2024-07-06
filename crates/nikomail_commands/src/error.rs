#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("Cache: {0}")]
	Cache(#[from] nikomail_cache::Error),

	#[error("SQLx: {0}")]
	Sqlx(#[from] sqlx::Error),

	#[error("Unknown")]
	Unknown
}

pub type Result<T> = core::result::Result<T, Error>;