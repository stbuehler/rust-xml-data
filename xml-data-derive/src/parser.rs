use std::ops::Deref;

use crate::element::{ElementInput, FieldAttribute, FieldChild, InnerInput};
use darling::util::SpannedValue;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens, TokenStreamExt};
use syn::Path;

mod state;

use state::State;

struct AttrExtractor<'a> {
	data: SpannedValue<&'a FieldAttribute>,
	xml_data_crate: &'a Path,
}

impl<'a> AttrExtractor<'a> {
	fn new(data: SpannedValue<&'a FieldAttribute>, xml_data_crate: &'a Path) -> Self {
		Self {
			data,
			xml_data_crate,
		}
	}
}

impl ToTokens for AttrExtractor<'_> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let Self {
			data,
			xml_data_crate,
		} = self;
		let value_type = data.value_type();
		let attr_key = &data.key;
		let ident = &data.ident;
		tokens.append_all(quote_spanned! {data.span()=>
			if #attr_key == key && self.#ident.is_none() {
				self.#ident = Some(#xml_data_crate::parser::#value_type::parse_value(value)?);
				return Ok(());
			}
		});
	}
}

enum ParseMode {
	Node,
	Text,
}

struct ChildExtractor<'a> {
	data: SpannedValue<&'a FieldChild>,
	success: &'a TokenStream,
	mode: ParseMode,
}

impl<'a> ChildExtractor<'a> {
	fn node(data: SpannedValue<&'a FieldChild>, success: &'a TokenStream) -> Self {
		Self {
			data,
			success,
			mode: ParseMode::Node,
		}
	}

	fn text(data: SpannedValue<&'a FieldChild>, success: &'a TokenStream) -> Self {
		Self {
			data,
			success,
			mode: ParseMode::Text,
		}
	}
}

impl ToTokens for ChildExtractor<'_> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let data = &self.data;
		let ident = &data.ident;
		let parse_success = self.success;

		tokens.append_all(match self.mode {
			ParseMode::Node => quote_spanned! {data.span()=>
				let parser = match self.#ident.parse_inner_node(tag, parser)? {
					InnerParseResult::Next(p) => p,
					InnerParseResult::Success => return #parse_success,
				};
			},
			ParseMode::Text => quote_spanned! {data.span()=>
				let text = match self.#ident.parse_inner_text(text)? {
					InnerParseResult::Next(t) => t,
					InnerParseResult::Success => return #parse_success,
				};
			},
		});
	}
}

pub fn derive_element_parser(input: &ElementInput) -> TokenStream {
	let ElementInput {
		ident,
		xml_data_crate,
		..
	} = input;

	let tag = input.tag();
	let success = quote! { Ok(()) };

	let state = State::new(ident, input.fields(), xml_data_crate);

	let attr_extractors = input
		.attrs()
		.map(|field| {
			AttrExtractor::new(
				SpannedValue::new(field.deref(), field.span()),
				xml_data_crate,
			)
		})
		.collect::<Vec<_>>();

	let child_nodes = input
		.elements()
		.map(|field| ChildExtractor::node(field, &success))
		.collect::<Vec<_>>();

	let child_text = input
		.elements()
		.map(|field| ChildExtractor::text(field, &success))
		.collect::<Vec<_>>();

	let handle_unknown_attribute = if input.ignore_unknown.attributes() {
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

	let handle_unknown_element = if input.ignore_unknown.elements() {
		quote! {
			parser.parse_element_state(&mut IgnoreElement)
		}
	} else {
		quote! {
			let _ = parser;
			return Err(errors::unexpected_element(tag));
		}
	};

	let handle_text = if input.ignore_unknown.text() {
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

pub fn derive_inner_parser(input: &InnerInput) -> TokenStream {
	let InnerInput {
		ident,
		xml_data_crate,
		..
	} = input;

	let success = quote!(Ok(#xml_data_crate::parser::InnerParseResult::Success));

	let state = State::new(ident, input.elements(), xml_data_crate);

	let child_nodes = input
		.elements()
		.map(|field| ChildExtractor::node(field, &success))
		.collect::<Vec<_>>();

	let child_text = input
		.elements()
		.map(|field| ChildExtractor::text(field, &success))
		.collect::<Vec<_>>();

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
