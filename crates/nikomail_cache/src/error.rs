#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("Model Error: {0}")]
	ModelError(#[from] nikomail_models::Error),
	
	#[error("SQLx: {0}")]
	Sqlx(#[from] sqlx::Error),

	#[error("Twilight HTTP: {0}")]
	TwilightHttp(#[from] twilight_http::Error),

	#[error("Twilight HTTP Deserialise: {0}")]
	TwilightHttpDeserialiser(#[from] twilight_http::response::DeserializeBodyError)
}

pub type Result<T> = core::result::Result<T, Error>;