//! Derive `xml-data` trait implementations
//!
//! Deriving supports the following attributes on the struct:
//! - `#[xml_data(tag("..."))]`: XML tag (only for deriving `Element`); defaults to struct name
//! - `#[xml_data(crate(...))]`: Name of `xml-data` crate in local scope; defaults to `xml_data`
//! - `#[xml_data(ignore_unknown)]`: Ignore unhandled/unknown attributes, inner nodes and inner text
//!   (only for deriving `Element`; `Inner` never fails for unknown data)
//!
//! And the following attributes on struct fields:
//! - `#[xml(attr)]: Mark field as attribute for containing XML element (only for deriving
//!   `Element`)
//! - `#[xml(attr("..."))]: Mark field as attribute for containing XML element with given key (only
//!   for deriving `Element`)
//! - `#[xml(attr_string)]: Mark field as string attribute (using `ValueString` instead of
//!   `ValueDefault`) for containing XML element (only for deriving `Element`)
//!
//! Multiple attributes can be combined like `#[xml(tag("..."), ignore_unknown)]`.
//!
#![warn(missing_docs)]
#![doc(html_root_url = "https://docs.rs/xml-data-derive/0.0.1")]

extern crate proc_macro;

mod element;
mod parser;
mod serialize;

use darling::FromDeriveInput;
use proc_macro::TokenStream;
use serde_derive_internals::Ctxt;
use syn::{parse_macro_input, DeriveInput};

use crate::element::{ElementInput, InnerInput, SerdeElementInput};

/// Derive `xml-data::{parser,serializer}::Element`
#[proc_macro_derive(Element, attributes(xml_data))]
pub fn derive_element(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	TokenStream::from(match ElementInput::from_derive_input(&input) {
		Ok(input) => {
			let mut output = serialize::derive_fixed_element(&input);
			output.extend(parser::derive_element_parser(&input));
			output
		}
		Err(e) => e.write_errors(),
	})
}

/// Derive `xml-data::serializer::Element`
#[proc_macro_derive(SerializerElement, attributes(xml_data, serde))]
pub fn derive_serializer_element(input: TokenStream) -> TokenStream {
	process_input(input, |v| serialize::s_derive_fixed(&v))
}

/// Derive `xml-data::parser::Element`
#[proc_macro_derive(ParserElement, attributes(xml_data))]
pub fn derive_parser_element(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	TokenStream::from(match ElementInput::from_derive_input(&input) {
		Ok(input) => parser::derive_element_parser(&input),
		Err(e) => e.write_errors(),
	})
}

/// Derive `xml-data::{parser,serializer}::Inner`
#[proc_macro_derive(Inner, attributes(xml_data))]
pub fn derive_inner(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	TokenStream::from(match InnerInput::from_derive_input(&input) {
		Ok(input) => {
			let mut output = serialize::derive_inner(&input);
			output.extend(parser::derive_inner_parser(&input));
			output
		}
		Err(e) => e.write_errors(),
	})
}

/// Derive `xml-data::serializer::Inner`
#[proc_macro_derive(SerializerInner, attributes(xml_data))]
pub fn derive_serializer_inner(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	TokenStream::from(match InnerInput::from_derive_input(&input) {
		Ok(input) => serialize::derive_inner(&input),
		Err(e) => e.write_errors(),
	})
}

/// Derive `xml-data::parser::Inner`
#[proc_macro_derive(ParserInner, attributes(xml_data))]
pub fn derive_parser_inner(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	TokenStream::from(match InnerInput::from_derive_input(&input) {
		Ok(input) => parser::derive_inner_parser(&input),
		Err(e) => e.write_errors(),
	})
}

fn process_input(
	input: TokenStream,
	to_tokens: impl FnOnce(SerdeElementInput<'_>) -> proc_macro2::TokenStream,
) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let ctxt = Ctxt::new();
	let parsed = match SerdeElementInput::new(&input, &ctxt) {
		Ok(v) => v,
		Err(e) => {
			let error = e.unwrap_or_else(|| {
				darling::Error::multiple(
					ctxt.check()
						.unwrap_err()
						.into_iter()
						.map(darling::Error::from)
						.collect::<Vec<_>>(),
				)
			});
			return error.write_errors().into();
		}
	};

	let output = TokenStream::from(to_tokens(parsed));
	let _ = ctxt.check();

	output
}
