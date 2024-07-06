use std::fmt::Display;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("Cache: {0}")]
	Cache(#[from] nikomail_cache::Error),

	#[error("Slash Arg: {0}")]
	SlashArgError(#[from] crate::macros::SlashArgError),

	#[error("SQLx: {0}")]
	Sqlx(#[from] sqlx::Error),

	#[error("Twilight HTTP: {0}")]
	TwilightHttp(#[from] twilight_http::Error),

	#[error("Unknown")]
	Unknown
}

#[derive(Debug)]
pub struct CoreError {
	pub kind: CoreErrorKind,
	pub source: Error
}

#[derive(Debug)]
pub enum CoreErrorKind {
	Autocomplete,
	Command,
	CommandArguments
}

impl std::error::Error for CoreError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		Some(&self.source)
	}
}

impl Display for CoreError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.source.fmt(f)
	}
}

pub type Result<T> = core::result::Result<T, Error>;