use crate::element::{
	CrateRoot, ElementInput, FieldAttribute, FieldBase, FieldChild, InnerInput, SField,
};
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens, TokenStreamExt};
use syn::{spanned::Spanned, Path};

mod state;

use state::State;

struct AttrExtractor<'a, P>(SField<'a, &'a FieldAttribute, P>);

impl<P: CrateRoot> ToTokens for AttrExtractor<'_, P> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let data = &self.0;
		let root = data.crate_root();
		let value_type = data.xml.value_type();
		let attr_key = data.name().deserialize_name();
		let ident = data.ident();

		tokens.append_all(quote_spanned! {data.span()=>
			if #attr_key == key && self.#ident.is_none() {
				self.#ident = Some(#root::parser::#value_type::parse_value(value)?);
				return Ok(());
			}
		});
	}
}

enum ParseMode {
	Node,
	Text,
}

/// Node and text extractors for fields represented as children in XML.
struct ChildExtractors<'a, P> {
	fields: Vec<SField<'a, &'a FieldChild, P>>,
	/// Tokens defining `Ok` return value
	success: TokenStream,
}

impl<'a, P: Copy> ChildExtractors<'a, P> {
	fn new(
		fields: impl IntoIterator<Item = SField<'a, &'a FieldChild, P>>,
		success: TokenStream,
	) -> Self {
		Self {
			fields: fields.into_iter().collect(),
			success,
		}
	}

	fn nodes(&'a self) -> Vec<ChildExtractor<'a, P>> {
		self.fields
			.iter()
			.map(|v| ChildExtractor::node(*v, &self.success))
			.collect()
	}

	fn text(&'a self) -> Vec<ChildExtractor<'a, P>> {
		self.fields
			.iter()
			.map(|v| ChildExtractor::text(*v, &self.success))
			.collect()
	}
}

struct ChildExtractor<'a, P> {
	data: SField<'a, &'a FieldChild, P>,
	success: &'a TokenStream,
	mode: ParseMode,
}

impl<'a, P> ChildExtractor<'a, P> {
	fn node(data: SField<'a, &'a FieldChild, P>, success: &'a TokenStream) -> Self {
		Self {
			data,
			success,
			mode: ParseMode::Node,
		}
	}

	fn text(data: SField<'a, &'a FieldChild, P>, success: &'a TokenStream) -> Self {
		Self {
			data,
			success,
			mode: ParseMode::Text,
		}
	}
}

impl<P: CrateRoot> ToTokens for ChildExtractor<'_, P> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let Self { data, success, .. } = self;
		let ident = data.ident();

		tokens.append_all(match self.mode {
			ParseMode::Node => quote_spanned! {data.span()=>
				let parser = match self.#ident.parse_inner_node(tag, parser)? {
					InnerParseResult::Next(p) => p,
					InnerParseResult::Success => return Ok(#success),
				};
			},
			ParseMode::Text => quote_spanned! {data.span()=>
				let text = match self.#ident.parse_inner_text(text)? {
					InnerParseResult::Next(t) => t,
					InnerParseResult::Success => return Ok(#success),
				};
			},
		});
	}
}

pub fn derive_element(input: &ElementInput) -> TokenStream {
	let xml_data_crate = input.crate_root();
	let ident = input.ident();
	let tag = input.tag();

	let state = State::new(ident, input.fields(), xml_data_crate);

	let attr_extractors = input.attrs().map(AttrExtractor).collect::<Vec<_>>();

	let children = ChildExtractors::new(input.children(), quote!(()));
	let child_nodes = children.nodes();
	let child_text = children.text();

	let handle_unknown_attribute = if input.ignore_unknown().attributes() {
		quote! {
			let _ = key;
			let _ = value;
			return Ok(());
		}
	} else {
		quote! {
			let _ = value;
			return Err(errors::unexpected_attribute(key));
		}
	};

	let handle_unknown_element = if input.ignore_unknown().elements() {
		quote! {
			parser.parse_element_state(&mut IgnoreElement)
		}
	} else {
		quote! {
			let _ = parser;
			return Err(errors::unexpected_element(tag));
		}
	};

	let handle_text = if input.ignore_unknown().text() {
		quote! {
			let _ = text;
		}
	} else {
		quote! {
			if !text.trim().is_empty() {
				return Err(errors::unexpected_text());
			}
		}
	};

	const_enclosure(
		xml_data_crate,
		quote! {
			#state

			impl FixedElementState for State {
				type Output = #ident;

				const TAG: &'static str = #tag;

				fn parse_element_attribute(&mut self, key: &str, value: Cow<'_, str>) -> Result<()> {
					#(#attr_extractors)*
					#handle_unknown_attribute
				}

				fn parse_element_inner_text(&mut self, text: Cow<'_, str>) -> Result<()> {
					#(#child_text)*
					#handle_text
					Ok(())
				}

				fn parse_element_inner_node<P: ElementParser>(&mut self, tag: &str, parser: P) -> Result<()> {
					#(#child_nodes)*
					#handle_unknown_element
				}

				fn parse_element_finish(self) -> Result<Self::Output> {
					::std::convert::TryFrom::try_from(self)
				}
			}

			impl Element for #ident {
				type ParseState = State;
			}
		},
	)
}

pub fn derive_inner(input: &InnerInput) -> TokenStream {
	let ident = input.ident();
	let xml_data_crate = input.crate_root();

	let state = State::new(ident, input.children(), xml_data_crate);

	let children = ChildExtractors::new(
		input.children(),
		quote!(#xml_data_crate::parser::InnerParseResult::Success),
	);
	let child_nodes = children.nodes();
	let child_text = children.text();

	const_enclosure(
		xml_data_crate,
		quote! {
			#state

			impl InnerState for State {
				type Output = #ident;

				fn parse_inner_text<'t>(&mut self, text: Cow<'t, str>) -> Result<InnerParseResult<Cow<'t, str>>> {
					#(#child_text)*
					Ok(InnerParseResult::Next(text))
				}

				fn parse_inner_node<P: ElementParser>(&mut self, tag: &str, parser: P) -> Result<InnerParseResult<P>> {
					#(#child_nodes)*
					Ok(InnerParseResult::Next(parser))
				}

				fn parse_inner_finish(self) -> Result<Self::Output> {
					::std::convert::TryFrom::try_from(self)
				}
			}

			impl Inner for #ident {
				type ParseState = State;
			}

			impl Inner for Option<#ident> {
				type ParseState = ParseInnerOptional<State>;
			}
		},
	)
}

fn const_enclosure(xml_data_crate: &Path, body: TokenStream) -> TokenStream {
	quote! {
		const _: () = {
			use std::borrow::Cow;

			use #xml_data_crate::{
				parser::{
					FixedElementState,
					ElementState,
					ElementParser,
					Element,
					ElementDefaultParseState,
					IgnoreElement,
					Inner,
					InnerState,
					InnerParseResult,
					ParseInnerOptional,
					Value,
					ValueString,
					ValueDefault,
				},
				errors,
				Result,
			};

			#body
		};
	}
}
