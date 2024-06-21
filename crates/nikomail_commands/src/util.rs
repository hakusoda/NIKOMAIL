use std::pin::Pin;
use serde::Serializer;
use twilight_model::{
	id::{ marker::GuildMarker, Id },
	channel::message::{
		component::{ Button, ActionRow, ButtonStyle, Component },
		ReactionType
	}
};

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub fn create_topic_button(guild_id: Option<Id<GuildMarker>>) -> Component {
	ActionRow {
		components: vec![
			Button {
				url: None,
				label: Some("Start new topic".into()),
				emoji: Some(ReactionType::Custom { animated: false, id: Id::new(1219234152709095424), name: Some("dap_me_up".into()) }),
				style: ButtonStyle::Primary,
				disabled: false,
				custom_id: Some(match guild_id {
					Some(x) => format!("create_topic_{x}"),
					None => "create_topic".into()
				})
			}.into()
		]
	}.into()
}

pub fn serialize_option_as_bool<T, S: Serializer>(value: &Option<T>, serialiser: S) -> Result<S::Ok, S::Error> {
	serialiser.serialize_bool(value.is_some())
}