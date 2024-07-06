use twilight_model::{
	channel::message::{ Component, MessageFlags },
	http::interaction::{ InteractionResponse, InteractionResponseType }
};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::Result;
use super::Context;

pub struct ReplyBuilder<'a> {
	context: &'a Context,
	data: InteractionResponseDataBuilder
}

impl<'a> ReplyBuilder<'a> {
	pub fn new(context: &'a Context, content: impl Into<String>) -> Self {
		Self {
			context,
			data: InteractionResponseDataBuilder::new()
				.content(content)
		}
	}

	pub fn components(mut self, components: impl IntoIterator<Item = Component>) -> Self {
		self.data = self.data.components(components);
		self
	}

	pub fn ephemeral(mut self) -> Self {
		self.data = self.data.flags(MessageFlags::EPHEMERAL);
		self
	}
}

impl IntoFuture for ReplyBuilder<'_> {
	type Output = Result<()>;
	type IntoFuture = impl Future<Output = Result<()>>;

	fn into_future(self) -> Self::IntoFuture {
		self.context.response(InteractionResponse {
			kind: InteractionResponseType::ChannelMessageWithSource,
			data: Some(self.data.build())
		})
	}
}