use tracing_error::SpanTrace;

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
	#[error("SQL Error: {0}")]
	SqlError(#[from] sqlx::Error)
}

#[derive(Debug)]
pub struct Error {
	pub kind: ErrorKind,
	pub context: SpanTrace
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{}", self.kind)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.kind.source()
    }
}

impl<E: Into<ErrorKind>> From<E> for Error {
    fn from(source: E) -> Self {
        Self {
			kind: Into::<ErrorKind>::into(source),
			context: SpanTrace::capture()
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;