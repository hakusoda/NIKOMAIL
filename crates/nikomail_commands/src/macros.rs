use async_trait::async_trait;
use twilight_model::{
	id::{
		marker::{ GuildMarker, ChannelMarker },
		Id
	},
	application::interaction::application_command::CommandOptionValue
};

use crate::{ Error, Result, Interaction };

#[macro_export]
macro_rules! parse_command_argument {
    // extracts #[choices(...)]
    /*($interaction:ident, $args:ident => $name:literal: INLINE_CHOICE $type:ty [$($index:literal: $value:literal),*]) => {
        if let Some(arg) = $args.iter().find(|arg| arg.name == $name) {
            let $crate::serenity_prelude::ResolvedValue::Integer(index) = arg.value else {
                return Err($crate::SlashArgError::new_command_structure_mismatch("expected integer, as the index for an inline choice parameter"));
            };
            match index {
                $( $index => $value, )*
                _ => return Err($crate::SlashArgError::new_command_structure_mismatch("out of range index for inline choice parameter")),
            }
        } else {
            return Err($crate::SlashArgError::new_command_structure_mismatch("a required argument is missing"));
        }
    };*/

    // extracts Option<T>
    ($interaction:ident, $args:ident => $name:literal: Option<$type:ty $(,)*>) => {
        if let Some(arg) = $args.iter().find(|arg| arg.name == $name) {
            Some($crate::extract_command_argument!($type, $interaction, &arg.value).await?)
        } else {
            None
        }
    };

    // extracts Vec<T>
    ($interaction:ident, $args:ident => $name:literal: Vec<$type:ty $(,)*>) => {
        match $crate::parse_command_argument!($interaction, $args => $name: Option<$type>) {
            Some(value) => vec![value],
            None => vec![],
        }
    };

    // extracts #[flag]
    ($interaction:ident, $args:ident => $name:literal: FLAG) => {
        $crate::parse_command_argument!($interaction, $args => $name: Option<bool>)
            .unwrap_or(false)
    };

    // exracts T
    ($interaction:ident, $args:ident => $name:literal: $($type:tt)*) => {
        $crate::parse_command_argument!($interaction, $args => $name: Option<$($type)*>).unwrap()
    };
}

#[macro_export]
macro_rules! parse_command_arguments {
    ($interaction:expr, $args:expr => $(
        ( $name:literal: $($type:tt)* )
    ),* $(,)? ) => {
        async {
			#[allow(unused_variables)]
			let (interaction, args) = ($interaction, $args);
            Ok::<_, $crate::error::Error>(( $(
                $crate::parse_command_argument!( interaction, args => $name: $($type)* ),
            )* ))
        }
    };
}

#[async_trait]
pub trait CommandArgumentExtractor<T>: Sized {
	async fn extract(
		self,
		interaction: &Interaction,
		value: &CommandOptionValue
	) -> Result<T>;
}

#[async_trait]
impl<T: ArgumentConvert + Send + Sync> CommandArgumentExtractor<T> for std::marker::PhantomData<T> {
	async fn extract(
		self,
		interaction: &Interaction,
		value: &CommandOptionValue
	) -> Result<T> {
		T::convert(
			interaction.guild_id,
			interaction.channel.as_ref().map(|x| x.id),
			value
		).await
	}
}

#[async_trait]
trait ArgumentConvert: Sized {
	async fn convert(guild_id: Option<Id<GuildMarker>>, channel_id: Option<Id<ChannelMarker>>, value: &CommandOptionValue) -> Result<Self>;
}

#[async_trait]
impl ArgumentConvert for String {
	async fn convert(_guild_id: Option<Id<GuildMarker>>, _channel_id: Option<Id<ChannelMarker>>, value: &CommandOptionValue) -> Result<Self> {
		match value {
			CommandOptionValue::String(x) => Ok(x.clone()),
			_ => Err(Error::Unknown)
		}
	}
}

#[macro_export]
macro_rules! extract_command_argument {
	($target:ty, $interaction:expr, $value:expr) => {{
		use $crate::macros::CommandArgumentExtractor as _;
		(&&std::marker::PhantomData::<$target>).extract(&$interaction, $value)
	}};
}