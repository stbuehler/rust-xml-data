use crate::{
	element::{
		Field,
		FieldAttribute,
		Meta,
	},
};
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};

pub fn build_parser(meta: &Meta, impl_element: bool) -> TokenStream {
	let Meta { xml_data_crate, name, tag, .. } = meta;

	let parse_success = if impl_element {
		quote! { Ok(()) }
	} else {
		quote! { Ok(InnerParseResult::Success) }
	};

	let state_fields: TokenStream = meta.fields.iter().map(|field| {
		let Field { name, span, ty, .. } = field;
		if let Some(attr) = &field.attr {
			if attr.optional {
				// already optional
				quote_spanned! {*span=>
					#name: #ty,
				}
			} else {
				quote_spanned! {*span=>
					#name: Option<#ty>,
				}
			}
		} else {
			// inner
			quote_spanned! {*span=>
				#name: <#ty as Inner>::ParseState,
			}
		}
	}).collect();
	let finish: TokenStream = meta.fields.iter().map(|field| {
		let Field { name, span, .. } = field;
		if let Some(attr) = &field.attr {
			let FieldAttribute { key: attr_key, .. } = attr;
			if attr.optional {
				quote_spanned! {*span=>
					#name: self.#name,
				}
			} else {
				quote_spanned! {*span=>
					#name: match self.#name {
						Some(v) => v,
						None => return Err(errors::missing_attribute(#attr_key)),
					},
				}
			}
		} else {
			// inner
			quote_spanned! {*span=>
				#name: self.#name.parse_inner_finish()?,
			}
		}
	}).collect();

	let el_attrs: TokenStream = meta.fields.iter().filter_map(|field| {
		if let Some(attr) = &field.attr {
			let FieldAttribute { key: attr_key, .. } = attr;
			let Field { name, span, .. } = field;
			let value_t = if attr.is_string {
				quote!(ValueString)
			} else {
				quote!(ValueDefault)
			};
			Some(quote_spanned! {*span=>
				if #attr_key == key && self.#name.is_none() {
					self.#name = Some(#value_t::parse_value(value)?);
					return Ok(())
				}
			})
		} else {
			None
		}
	}).collect();
	let el_inner_node: TokenStream = meta.fields.iter().filter_map(|field| {
		if field.attr.is_none() {
			let Field { name, span, .. } = field;
			Some(quote_spanned! {*span=>
				let parser = match self.#name.parse_inner_node(tag, parser)? {
					InnerParseResult::Next(p) => p,
					InnerParseResult::Success => return #parse_success,
				};
			})
		} else {
			None
		}
	}).collect();
	let el_inner_text: TokenStream = meta.fields.iter().filter_map(|field| {
	if field.attr.is_none() {
			let Field { name, span, .. } = field;
			Some(quote_spanned! {*span=>
				let text = match self.#name.parse_inner_text(text)? {
					InnerParseResult::Next(t) => t,
					InnerParseResult::Success => return #parse_success,
				};
			})
		} else {
			None
		}
	}).collect();

	let handle_unknown_attribute = if meta.ignore_unknown_attribute {
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
	let handle_unknown_element = if meta.ignore_unknown_attribute {
		quote! {
			parser.parse_element_state(&mut IgnoreElement)
		}
	} else {
		quote! {
			let _ = parser;
			return Err(errors::unexpected_element(tag));
		}
	};
	let handle_text = if meta.ignore_text {
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

	let actual_impl = if impl_element {
		quote! {
			impl FixedElementState for State {
				type Output = #name;

				const TAG: &'static str = #tag;

				fn parse_element_attribute(&mut self, key: &str, value: Cow<'_, str>) -> Result<()> {
					#el_attrs
					#handle_unknown_attribute
				}

				fn parse_element_inner_text(&mut self, text: Cow<'_, str>) -> Result<()> {
					#el_inner_text
					#handle_text
					Ok(())
				}

				fn parse_element_inner_node<P: ElementParser>(&mut self, tag: &str, parser: P) -> Result<()> {
					#el_inner_node
					#handle_unknown_element
				}

				fn parse_element_finish(self) -> Result<Self::Output> {
					Ok(#name {
						#finish
					})
				}
			}

			impl Element for #name {
				type ParseState = State;
			}
		}
	} else {
		quote! {
			impl InnerState for State {
				type Output = #name;

				fn parse_inner_text<'t>(&mut self, text: Cow<'t, str>) -> Result<InnerParseResult<Cow<'t, str>>> {
					#el_inner_text
					Ok(InnerParseResult::Next(text))
				}

				fn parse_inner_node<P: ElementParser>(&mut self, tag: &str, parser: P) -> Result<InnerParseResult<P>> {
					#el_inner_node
					Ok(InnerParseResult::Next(parser))
				}

				fn parse_inner_finish(self) -> Result<Self::Output> {
					Ok(#name {
						#finish
					})
				}
			}

			impl Inner for #name {
				type ParseState = State;
			}

			impl Inner for Option<#name> {
				type ParseState = ParseInnerOptional<State>;
			}
		}
	};

	quote! {
		const _: () = {
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
			use std::borrow::Cow;
			use std::string::ToString;

			#[derive(Default)]
			pub struct State {
				#state_fields
			}

			#actual_impl
		};
	}
}
