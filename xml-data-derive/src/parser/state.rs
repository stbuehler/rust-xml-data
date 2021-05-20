use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens, TokenStreamExt};
use syn::{spanned::Spanned, Ident, Path};

use crate::element::{Field, FieldAttribute, FieldChild};

pub(super) struct State<'a, T> {
	ident: &'a Ident,
	fields: Vec<T>,
	xml_data_crate: &'a Path,
}

impl<'a, T> State<'a, T> {
	pub(super) fn new(
		ident: &'a Ident,
		fields: impl IntoIterator<Item = T>,
		xml_data_crate: &'a Path,
	) -> Self {
		Self {
			ident,
			fields: fields.into_iter().collect(),
			xml_data_crate,
		}
	}
}

impl<'a, T> ToTokens for State<'a, T>
where
	T: Copy,
	FieldDeclaration<'a, T>: ToTokens,
	FieldInitializer<'a, T>: ToTokens,
{
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let Self {
			ident,
			fields,
			xml_data_crate,
		} = self;

		let (declarations, initializers): (Vec<_>, Vec<_>) = fields
			.iter()
			.map(|&field| {
				(
					FieldDeclaration::new(field, xml_data_crate),
					FieldInitializer::new(field, xml_data_crate),
				)
			})
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
pub(super) struct FieldDeclaration<'a, T> {
	data: T,
	xml_data_crate: &'a Path,
}

impl<'a, T> FieldDeclaration<'a, T> {
	fn new(data: T, xml_data_crate: &'a Path) -> Self {
		Self {
			data,
			xml_data_crate,
		}
	}

	fn new_value<U>(&'a self, value: U) -> FieldDeclaration<'a, U> {
		FieldDeclaration::new(value, self.xml_data_crate)
	}
}

impl<'a> ToTokens for FieldDeclaration<'a, &'a Field> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		match &self.data {
			Field::Attribute(attr) => self.new_value(attr).to_tokens(tokens),
			Field::Child(child) => self.new_value(child).to_tokens(tokens),
		}
	}
}

impl<'a> ToTokens for FieldDeclaration<'a, &'a FieldAttribute> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let Self { data, .. } = self;
		let ident = &data.ident;
		let ty = &data.ty;
		tokens.append_all(if data.optional {
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

impl<'a> ToTokens for FieldDeclaration<'a, &'a FieldChild> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let Self {
			data,
			xml_data_crate,
		} = self;
		let ident = &data.ident;
		let ty = &data.ty;

		tokens.append_all(quote_spanned! {data.span()=>
			#ident: <#ty as #xml_data_crate::parser::Inner>::ParseState,
		});
	}
}

/// Wrapper for initializing a field in the deriving type from a field in the builder type.
pub(super) struct FieldInitializer<'a, T> {
	data: T,
	xml_data_crate: &'a Path,
}

impl<'a, T> FieldInitializer<'a, T> {
	fn new(data: T, xml_data_crate: &'a Path) -> Self {
		Self {
			data,
			xml_data_crate,
		}
	}

	fn new_value<U>(&'a self, value: U) -> FieldInitializer<'a, U> {
		FieldInitializer::new(value, self.xml_data_crate)
	}
}

impl<'a> ToTokens for FieldInitializer<'a, &'a Field> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		match self.data {
			Field::Attribute(attr) => self.new_value(attr).to_tokens(tokens),
			Field::Child(child) => self.new_value(child).to_tokens(tokens),
		}
	}
}

impl<'a> ToTokens for FieldInitializer<'a, &'a FieldAttribute> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let Self {
			data,
			xml_data_crate,
		} = self;
		let ident = &data.ident;
		tokens.append_all(if data.optional {
			quote_spanned! {data.span()=>
				#ident: state.#ident,
			}
		} else {
			let attr_key = &data.key;
			quote_spanned! {data.span()=>
				#ident: match state.#ident {
					Some(v) => v,
					None => {
						return Err(#xml_data_crate::errors::missing_attribute(#attr_key));
					}
				},
			}
		});
	}
}

impl<'a> ToTokens for FieldInitializer<'a, &'a FieldChild> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let ident = &self.data.ident;

		tokens.append_all(quote_spanned! {self.data.span()=>
			#ident: state.#ident.parse_inner_finish()?,
		});
	}
}
