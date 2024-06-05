use twilight_gateway::{ Event, MessageSender };

use super::event;
use crate::Result;

pub struct Context;

impl Context {
	pub fn new(_message_sender: MessageSender) -> Self {
		Self {}
	}

	pub async fn handle_event(self: crate::Context, event: Event) -> Result<()> {
		let event_kind = event.kind();
		tracing::info!("handle_event {event_kind:?}");
		
		tokio::spawn(async move {
			tracing::debug!("handle_event {event_kind:?} >");

			if let Err(error) =  match event {
				Event::InteractionCreate(x) => event::interaction::interaction_create(self, *x).await,
				Event::MessageCreate(x) => event::message::message_create(*x).await,
				Event::MessageUpdate(x) => event::message::message_update(*x).await,
				Event::MessageDelete(x) => event::message::message_delete(x).await,
				Event::ReactionAdd(x) => event::reaction::reaction_add(*x).await,
				Event::ThreadCreate(x) => event::thread::thread_create(*x),
				Event::ThreadUpdate(x) => event::thread::thread_update(*x).await,
				Event::ThreadDelete(x) => event::thread::thread_delete(x).await,
				Event::TypingStart(x) => event::misc::typing_start(*x).await,
				_ => Ok(())
			} {
				println!("error occurred in event handler! {error}");
			}

			tracing::debug!("handle_event {event_kind:?} <");
		});
		Ok(())
	}
}