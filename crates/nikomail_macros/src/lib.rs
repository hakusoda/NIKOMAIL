use syn::spanned::Spanned;
use quote::format_ident;
use darling::FromMeta;
use proc_macro::TokenStream;

mod util;

#[derive(Debug, Default, FromMeta)]
#[darling(default)]
struct CommandArgs {
	user: bool,
	slash: bool,
	rename: Option<String>,
	message: bool,
	#[darling(rename = "context", multiple)]
	contexts: Vec<String>,
	description: Option<String>,
	default_member_permissions: Option<String>
}

#[derive(Default, Debug, darling::FromMeta)]
#[darling(default)]
struct ParamArgs {
	rename: Option<String>,
	autocomplete: Option<syn::Path>
}

struct CommandOption {
	name: String,
	kind: syn::Type,
	blah: proc_macro2::TokenStream
}

fn wrap_option_to_string<T: quote::ToTokens>(literal: Option<T>) -> syn::Expr {
    match literal {
        Some(literal) => syn::parse_quote! { Some(#literal.to_string()) },
        None => syn::parse_quote! { None },
    }
}

fn create_command(args: TokenStream, mut function: syn::ItemFn) -> Result<TokenStream, darling::Error> {
	let args = darling::ast::NestedMeta::parse_meta_list(args.into())?;
    let args = <CommandArgs as darling::FromMeta>::from_list(&args)?;
	if !args.user && !args.slash && !args.message {
		return Err(syn::Error::new(function.sig.span(), "command must specify either user, slash, or message").into());
	}

	let function_name = function
        .sig
        .ident
        .to_string()
        .trim_start_matches("r#")
        .to_string();

	let function_ident = std::mem::replace(&mut function.sig.ident, syn::parse_quote! { inner });
    let function_generics = &function.sig.generics;
    let function_visibility = &function.vis;

	let contexts: Vec<syn::Expr> = args.contexts.into_iter().map(|x| match x.as_str() {
		"guild" => syn::parse_quote! { crate::InteractionContextKind::Guild },
		"bot_dm" => syn::parse_quote! { crate::InteractionContextKind::BotDM },
		"private_channel" => syn::parse_quote! { crate::InteractionContextKind::PrivateChannel },
		_ => panic!("invalid context, must specify either guild, bot_dm, or private_channel")
	}).collect();
	let is_user = args.user;
	let is_slash = args.slash;
	let is_message = args.message;
	let description = wrap_option_to_string(args.description);
	let default_member_permissions = wrap_option_to_string(args.default_member_permissions);

	let mut parameters: Vec<CommandOption> = vec![];
    for command_param in function.sig.inputs.iter_mut().skip(2) {
       	let pattern = match command_param {
            syn::FnArg::Typed(x) => x,
            syn::FnArg::Receiver(r) => {
                return Err(syn::Error::new(r.span(), "self argument is invalid here").into());
            }
        };

        let attrs: Vec<_> = pattern
            .attrs
            .drain(..)
            .map(|attr| darling::ast::NestedMeta::Meta(attr.meta))
            .collect();
        let attrs = <ParamArgs as darling::FromMeta>::from_list(&attrs)?;

		let name = if let Some(rename) = &attrs.rename {
            rename.clone()
        } else if let syn::Pat::Ident(ident) = &*pattern.pat {
            ident.ident.to_string().trim_start_matches("r#").into()
        } else {
            let message = "#[rename = \"...\"] must be specified for pattern parameters";
            return Err(syn::Error::new(pattern.pat.span(), message).into());
        };

		let autocomplete = match attrs.autocomplete {
			Some(autocomplete_fn) => quote::quote! {
				Some(|ctx, interaction, partial| Box::pin(async move {
					#autocomplete_fn(ctx, interaction, partial).await
				}))
			},
			None => quote::quote! { None }
		};

		let kind = &pattern.ty;
		let required = util::extract_type_parameter("Option", kind).is_none();
        parameters.push(CommandOption {
			name: name.clone(),
			kind: *kind.clone(),
			blah: quote::quote! {
				crate::CommandOption {
					name: #name.to_string(),
					kind: crate::CommandOptionKind::String,
					required: #required,
					description: None,
					autocomplete: #autocomplete
				}
			}
		});
    }

	let param_identifiers = (0..parameters.len())
        .map(|i| format_ident!("nikomail_param_{i}"))
        .collect::<Vec<_>>();
    let param_names = parameters.iter().map(|p| &p.name).collect::<Vec<_>>();

    let param_types = parameters
        .iter()
        .map(|p| {
			let t = &p.kind;
            /*if p.args.flag {
                quote::quote! { FLAG }
            } else if let Some(choices) = &p.args.choices {
                let choice_indices = (0..choices.0.len()).map(syn::Index::from);
                let choice_vals = &choices.0;
                quote::quote! { INLINE_CHOICE #t [#(#choice_indices: #choice_vals),*] }
            } else {
                quote::quote! { #t }
            }*/
			quote::quote! { #t }
        })
        .collect::<Vec<_>>();

	let handler = quote::quote! {
		|context, interaction| Box::pin(async move {
			let ( #( #param_identifiers, )* ) = crate::parse_command_arguments!(
                &interaction, interaction.options() =>
                #( (#param_names: #param_types), )*
            ).await?;

			inner(context, interaction, #( #param_identifiers, )*).await
		})
	};

	let name = match args.rename {
		Some(x) => x,
		None => function_name
	};
	let options: Vec<proc_macro2::TokenStream> = parameters.into_iter().map(|x| x.blah).collect();
	Ok(TokenStream::from(quote::quote! {
		#function_visibility fn #function_ident #function_generics() -> crate::Command {
            #function
			crate::Command {
				name: #name.to_string(),
				options: vec![ #( #options ),* ],
				contexts: vec![ #( #contexts ),* ],
				handler: #handler,
				is_user: #is_user,
				is_slash: #is_slash,
				is_message: #is_message,
				description: #description,
				default_member_permissions: #default_member_permissions
			}
		}
	}))
}

#[proc_macro_attribute]
pub fn command(args: TokenStream, function: TokenStream) -> TokenStream {
	let function = syn::parse_macro_input!(function as syn::ItemFn);
	match create_command(args, function) {
		Ok(x) => x,
		Err(x) => x.write_errors().into()
	}
}