use serde::{ Serialize, Deserialize };
use chrono::{ Utc, DateTime };
use serde_repr::*;
use nikomail_commands::{
	command::CommandResponse,
	commands::COMMANDS,
	Interaction
};
use nikomail_util::DISCORD_INTERACTION_CLIENT;
use twilight_model::{
	http::interaction::{ InteractionResponse, InteractionResponseData, InteractionResponseType },
	channel::message::MessageFlags,
	application::interaction::{
		application_command::CommandOptionValue,
		Interaction as TwilightInteraction, InteractionData
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

async fn parse_interaction(interaction: Interaction) -> Result<InteractionResponse> {
	match interaction.data.as_ref().unwrap() {
		InteractionData::ApplicationCommand(data) => {
			if let Some(command) = COMMANDS.iter().find(|x| x.name == data.name) {
				for option in data.options.iter() {
					if let CommandOptionValue::Focused(partial, _kind) = &option.value {
						let partial = partial.clone();
						let command_option = command.options.iter().find(|x| x.name == option.name).unwrap();
						return Ok(InteractionResponse {
							kind: InteractionResponseType::ApplicationCommandAutocompleteResult,
							data: Some(InteractionResponseData {
								choices: Some(command_option.autocomplete.unwrap()(interaction, partial).await?),
								..Default::default()
							})
						});
					}
				}
				let response = match (command.handler)(interaction).await {
					Ok(x) => x,
					Err(error) => {
						println!("{error}");
						return Err(error.into());
					}
				};
				Ok(match response {
					CommandResponse::Message { flags, content, components } =>
						InteractionResponse {
							kind: InteractionResponseType::ChannelMessageWithSource,
							data: Some(InteractionResponseData {
								flags,
								content,
								components,
								..Default::default()
							})
						},
					CommandResponse::Defer =>
						InteractionResponse {
							kind: InteractionResponseType::DeferredChannelMessageWithSource,
							data: Some(InteractionResponseData {
								flags: Some(MessageFlags::EPHEMERAL),
								..Default::default()
							})
						}
				})
			} else {
				Ok(InteractionResponse {
					kind: InteractionResponseType::ChannelMessageWithSource,
					data: Some(InteractionResponseData {
						content: Some("<:niko_look_left:1227198516590411826> erm... this command hasn't been implemented yet...".into()),
						..Default::default()
					})
				})
			}
		},
		_ => Ok(InteractionResponse {
			kind: InteractionResponseType::ChannelMessageWithSource,
			data: Some(InteractionResponseData {
				content: Some("<:niko_look_left:1227198516590411826> erm... unsure what you're trying to do but i don't know how to handle this yet!".into()),
				..Default::default()
			})
		})
	}
}

#[tracing::instrument(level = "trace")]
pub async fn handle_interaction(interaction: TwilightInteraction) -> Result<()> {
	let id = interaction.id;
	let token = interaction.token.clone();
	/*if let Some(user) = interaction.author() {
		DISCORD_MODELS.users.insert(user.id, user.clone().into());
	}*/

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

	let response = parse_interaction(interaction).await?;
	DISCORD_INTERACTION_CLIENT.create_response(id, &token, &response)
		.await?;

	Ok(())
}