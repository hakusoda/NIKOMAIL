use serde::{ Serialize, Deserialize };
use chrono::{ Utc, DateTime };
use serde_repr::*;
use twilight_model::{
	id::{
		marker::{ UserMarker, GuildMarker, ApplicationMarker, InteractionMarker },
		Id
	},
	http::interaction::{ InteractionResponse, InteractionResponseData, InteractionResponseType },
	guild::Permissions,
	channel::{ message::MessageFlags, Channel, Message },
	application::interaction::{
		application_command::{ CommandDataOption, CommandOptionValue },
		Interaction as TwilightInteraction, InteractionData, InteractionType
	}
};

use crate::{
	discord::INTERACTION,
	Result, Context, CommandResponse
};
use super::commands::COMMANDS;

#[derive(Clone, Debug, PartialEq)]
pub struct Interaction {
    pub app_permissions: Option<Permissions>,
    pub application_id: Id<ApplicationMarker>,
    pub channel: Option<Channel>,
    pub data: Option<InteractionData>,
    pub guild_id: Option<Id<GuildMarker>>,
    pub guild_locale: Option<String>,
    pub id: Id<InteractionMarker>,
    pub kind: InteractionType,
    pub locale: Option<String>,
    pub message: Option<Message>,
    pub token: String,
    pub user_id: Option<Id<UserMarker>>,
}

impl Interaction {
	pub fn options(&self) -> Vec<&CommandDataOption> {
		match &self.data {
			Some(InteractionData::ApplicationCommand(x)) => x.options.iter().collect(),
			_ => vec![]
		}
	}
	/*pub async fn user(&self) -> Result<Option<Ref<'_, Id<UserMarker>, CachedUser>>> {
		Ok(if let Some(user_id) = self.user_id {
			Some(DISCORD_MODELS.user(user_id).await?)
		} else { None })
	}*/

	/*pub async fn member(&self) -> Result<Option<Ref<'static, (Id<GuildMarker>, Id<UserMarker>), CachedMember>>> {
		Ok(if let Some(user_id) = self.user_id && let Some(guild_id) = self.guild_id {
			Some(CACHE.discord.member(guild_id, user_id).await?)
		} else { None })
	}*/
}

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

async fn parse_interaction(context: Context, interaction: Interaction) -> Result<InteractionResponse> {
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
								choices: Some(command_option.autocomplete.unwrap()(context, interaction, partial).await?),
								..Default::default()
							})
						});
					}
				}
				let response = match (command.handler)(context, interaction).await {
					Ok(x) => x,
					Err(error) => {
						println!("{error}");
						return Err(error);
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

#[tracing::instrument(skip(context), level = "trace")]
pub async fn handle_interaction(context: Context, interaction: TwilightInteraction) -> Result<()> {
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

	let response = parse_interaction(context, interaction).await?;
	INTERACTION.create_response(id, &token, &response).await?;

	Ok(())
}