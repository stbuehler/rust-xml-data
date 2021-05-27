use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens, TokenStreamExt};
use serde_derive_internals::attr;
use syn::{spanned::Spanned, Ident, Path};

use crate::element::{
	CrateRoot, Field, FieldAttribute, FieldBase, FieldChild, SField, StructDefault,
};

pub(super) struct State<'a, T, P> {
	ident: &'a Ident,
	fields: Vec<SField<'a, T, P>>,
	xml_data_crate: &'a Path,
}

impl<'a, T, P> State<'a, T, P> {
	pub(super) fn new(
		ident: &'a Ident,
		fields: impl IntoIterator<Item = SField<'a, T, P>>,
		xml_data_crate: &'a Path,
	) -> Self {
		Self {
			ident,
			fields: fields.into_iter().collect(),
			xml_data_crate,
		}
	}
}

impl<'a, T, P> ToTokens for State<'a, T, P>
where
	T: Copy,
	P: Copy,
	FieldDeclaration<'a, T, P>: ToTokens,
	FieldInitializer<'a, T, P>: ToTokens,
{
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let Self {
			ident,
			fields,
			xml_data_crate,
		} = self;

		let (declarations, initializers): (Vec<_>, Vec<_>) = fields
			.iter()
			.map(|&field| (FieldDeclaration(field), FieldInitializer(field)))
			.unzip();

		tokens.append_all(quote! {
			#[doc(hidden)]
			#[derive(Default)]
			pub struct State {
				#(#declarations)*
			}

			impl ::std::convert::TryFrom<State> for #ident {
				type Error = #xml_data_crate::Error;

				fn try_from(state: State) -> #xml_data_crate::Result<Self> {
					Ok(Self {
						#(#initializers)*
					})
				}
			}
		})
	}
}

/// Initial declaration of a field, where its type is determined.
pub(super) struct FieldDeclaration<'a, T, P>(SField<'a, T, P>);

impl<'a, P: Copy + CrateRoot> ToTokens for FieldDeclaration<'a, &'a Field, P> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		if let Some(v) = self.0.as_field_attr().map(FieldDeclaration) {
			v.to_tokens(tokens)
		} else if let Some(v) = self.0.as_field_child().map(FieldDeclaration) {
			v.to_tokens(tokens)
		}
	}
}

impl<'a, P: CrateRoot> ToTokens for FieldDeclaration<'a, &'a FieldChild, P> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let data = &self.0;
		let root = data.crate_root();
		let ident = data.ident();
		let ty = data.ty();

		tokens.append_all(quote_spanned! {data.span()=>
			#ident: <#ty as #root::parser::Inner>::ParseState,
		});
	}
}

impl<'a, P> ToTokens for FieldDeclaration<'a, &'a FieldAttribute, P> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let data = &self.0;
		let ident = data.ident();
		let ty = data.ty();
		tokens.append_all(if data.is_optional() {
			quote_spanned! {data.span()=>
				#ident: #ty,
			}
		} else {
			quote_spanned! {data.span()=>
				#ident: Option<#ty>,
			}
		});
	}
}

/// Wrapper for initializing a field in the deriving type from a field in the builder type.
pub(super) struct FieldInitializer<'a, T, P>(SField<'a, T, P>);

impl<'a, P: Copy + CrateRoot + StructDefault> ToTokens for FieldInitializer<'a, &'a Field, P> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		if let Some(v) = self.0.as_field_attr().map(FieldInitializer) {
			v.to_tokens(tokens)
		} else if let Some(v) = self.0.as_field_child().map(FieldInitializer) {
			v.to_tokens(tokens)
		}
	}
}

impl<'a, P: CrateRoot> ToTokens for FieldInitializer<'a, &'a FieldChild, P> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let ident = &self.0.ident();

		tokens.append_all(quote_spanned! {self.0.span()=>
			#ident: state.#ident.parse_inner_finish()?,
		});
	}
}

impl<'a, P: CrateRoot + StructDefault> ToTokens for FieldInitializer<'a, &'a FieldAttribute, P> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let data = &self.0;
		let root = data.crate_root();
		let ident = data.ident();

		tokens.append_all(if data.is_optional() {
			quote_spanned! {data.span()=>
				#ident: state.#ident,
			}
		} else {
			let attr_key = data.name().deserialize_name();
			let fallback = match data.serde.attrs.default() {
				attr::Default::Default => {
					quote_spanned!(data.span()=>::std::default::Default::default(),)
				}
				attr::Default::Path(path) => quote!(#path(),),
				// If there's no field-level default, use the struct-level default if defined
				attr::Default::None if !data.struct_default().is_none() => {
					quote!(struct_default.#ident)
				}
				// If there's no field or struct default, but the attribute is optional, use `None`
				attr::Default::None if data.is_optional() => quote!(None),
				// Otherwise, return an error.
				attr::Default::None => {
					quote_spanned! {data.span()=>{
						return Err(#root::errors::missing_attribute(#attr_key));
					}}
				}
			};

			quote_spanned! {data.span()=>
				#ident: match state.#ident {
					Some(v) => v,
					None => #fallback
				},
			}
		});
	}
}
