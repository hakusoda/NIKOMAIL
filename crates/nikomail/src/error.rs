#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("Cache: {0}")]
	Cache(#[from] nikomail_cache::Error),

	#[error("Command: {0}")]
	Command(#[from] nikomail_commands::Error),

	#[error("Model: {0}")]
	Model(#[from] nikomail_models::Error),

	#[error("Reqwest: {0}")]
	Reqwest(#[from] reqwest::Error),

	#[error("JSON: {0}")]
	Json(#[from] serde_json::Error),

	#[error("Twilight HTTP Deserialise: {0}")]
	TwilightHttpDeserialise(#[from] twilight_http::response::DeserializeBodyError),

	#[error("Twilight HTTP: {0}")]
	TwilightHttp(#[from] twilight_http::Error),

	#[error("SQLx: {0}")]
	Sqlx(#[from] sqlx::Error)
}

pub type Result<T> = core::result::Result<T, Error>;