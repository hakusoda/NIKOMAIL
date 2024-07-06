pub fn extract_type_parameter<'a>(outer_type: &str, t: &'a syn::Type) -> Option<&'a syn::Type> {
	if let syn::Type::Path(path) = t {
		if path.path.segments.len() == 1 {
			let path = &path.path.segments[0];
			if path.ident == outer_type {
				if let syn::PathArguments::AngleBracketed(generics) = &path.arguments {
					if generics.args.len() == 1 {
						if let syn::GenericArgument::Type(t) = &generics.args[0] {
							return Some(t);
						}
					}
				}
			}
		}
	}
	None
}

#[derive(Debug)]
pub struct List<T>(pub Vec<T>);

impl<T: darling::FromMeta> darling::FromMeta for List<T> {
	fn from_list(items: &[::darling::ast::NestedMeta]) -> darling::Result<Self> {
		items
			.iter()
			.map(|item| T::from_nested_meta(item))
			.collect::<darling::Result<Vec<T>>>()
			.map(Self)
	}
}
impl<T> Default for List<T> {
	fn default() -> Self {
		Self(Default::default())
	}
}