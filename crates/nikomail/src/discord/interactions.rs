use chrono::{ Utc, DateTime };
use nikomail_commands_core::{ Context, Interaction };
use nikomail_util::DISCORD_INTERACTION_CLIENT;
use serde::{ Serialize, Deserialize };
use serde_repr::*;
use twilight_model::{
	http::interaction::{ InteractionResponse, InteractionResponseData, InteractionResponseType },
	application::interaction::{
		application_command::CommandOptionValue,
		Interaction as TwilightInteraction
	}
};

use crate::Result;

#[derive(Deserialize_repr, Debug)]
#[repr(u8)]
pub enum ApplicationCommandKind {
	ChatInput = 1,
	User,
	Message
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Embed {
	pub url: Option<String>,
	pub title: Option<String>,
	pub author: Option<EmbedAuthor>,
	pub fields: Option<Vec<EmbedField>>,
	pub footer: Option<EmbedFooter>,
	pub timestamp: Option<DateTime<Utc>>,
	pub description: Option<String>
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct EmbedAuthor {
	pub url: Option<String>,
	pub name: Option<String>,
	pub icon_url: Option<String>
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EmbedField {
	pub name: String,
	pub value: String,
	pub inline: Option<bool>
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EmbedFooter {
	pub text: String,
	pub icon_url: Option<String>
}

async fn parse_interaction(context: Context) -> Result<()> {
	if let Some((context, command)) = nikomail_commands::commands::process_context(context) {
		for option in context.options.iter() {
			if let CommandOptionValue::Focused(partial, _kind) = &option.value {
				let interaction_id = context.interaction.id;
				let interaction_token = context.interaction.token.clone();

				let partial = partial.clone();
				let command_option = command.options
					.iter()
					.find(|x| x.name == option.name)
					.unwrap();
				let choices = command_option.autocomplete.unwrap()(context, partial).await?;
				DISCORD_INTERACTION_CLIENT.create_response(interaction_id, &interaction_token, &InteractionResponse {
					kind: InteractionResponseType::ApplicationCommandAutocompleteResult,
					data: Some(InteractionResponseData {
						choices: Some(choices),
						..Default::default()
					})
				}).await?;

				return Ok(());
			}
		}

		match (command.handler)(context).await {
			Ok(x) => x,
			Err(error) => {
				println!("{error}");
				return Err(error.into());
			}
		}
	} else {
		println!("command no tfoun");
	}

	Ok(())
}

#[tracing::instrument(level = "trace")]
pub async fn handle_interaction(interaction: TwilightInteraction) -> Result<()> {
	let interaction = Interaction {
		app_permissions: interaction.app_permissions,
		application_id: interaction.application_id,
		channel: interaction.channel,
		data: interaction.data,
		guild_id: interaction.guild_id,
		guild_locale: interaction.guild_locale,
		id: interaction.id,
		kind: interaction.kind,
		locale: interaction.locale,
		message: interaction.message,
		token: interaction.token,
		user_id: match interaction.member {
			Some(member) => member.user.map(|x| x.id),
			None => interaction.user.map(|x| x.id)
		}
	};

	parse_interaction(Context::new(interaction))
		.await?;

	Ok(())
}