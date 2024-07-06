use twilight_model::{
	channel::message::{
		component::{ Button, ActionRow, ButtonStyle, Component },
		ReactionType
	},
	id::{
		marker::GuildMarker,
		Id
	}
};

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