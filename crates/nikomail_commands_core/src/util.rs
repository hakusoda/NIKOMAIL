use std::pin::Pin;
use serde::Serializer;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub fn serialize_option_as_bool<T, S: Serializer>(value: &Option<T>, serialiser: S) -> Result<S::Ok, S::Error> {
	serialiser.serialize_bool(value.is_some())
}