#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("SQLx: {0}")]
	Sqlx(#[from] sqlx::Error)
}

pub type Result<T> = core::result::Result<T, Error>;