use twilight_gateway::Event;

use crate::Result;

pub mod misc;
pub mod thread;
pub mod message;
pub mod reaction;
pub mod interaction;

pub fn handle_event(event: Event) {
	let event_kind = event.kind();
	tracing::info!("handle_event {event_kind:?}");

	if let Err(error) = match event {
		Event::InteractionCreate(x) => spawn(interaction::interaction_create(*x)),
		Event::MessageCreate(x) => spawn(message::message_create(*x)),
		Event::MessageUpdate(x) => spawn(message::message_update(*x)),
		Event::MessageDelete(x) => spawn(message::message_delete(x)),
		Event::ReactionAdd(x) => spawn(reaction::reaction_add(*x)),
		Event::ThreadCreate(x) => thread::thread_create(*x),
		Event::ThreadUpdate(x) => spawn(thread::thread_update(*x)),
		Event::ThreadDelete(x) => spawn(thread::thread_delete(x)),
		Event::TypingStart(x) => spawn(misc::typing_start(*x)),
		_ => Ok(())
	} {
		println!("error occurred in event handler! {error}");
	}
}

fn spawn<F: Future<Output = Result<()>> + Send + 'static>(future: F) -> Result<()> {
	tokio::spawn(async move {
		if let Err(error) = future.await {
			println!("error occurred in async event handler! {error}");
		}
	});

	Ok(())
}