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

mod attributes;
mod element;
mod parser;
mod serialize;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

/// Derive `xml-data::{parser,serializer}::Element`
#[proc_macro_derive(Element, attributes(xml_data))]
pub fn derive_element(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let meta = element::Meta::parse_meta(&input, true);

	let mut output = serialize::build_serialize(&meta, true);
	output.extend(parser::build_parser(&meta, true));

	TokenStream::from(output)
}

/// Derive `xml-data::serializer::Element`
#[proc_macro_derive(SerializerElement, attributes(xml_data))]
pub fn derive_serializer_element(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let meta = element::Meta::parse_meta(&input, true);

	let output = serialize::build_serialize(&meta, true);

	TokenStream::from(output)
}

/// Derive `xml-data::parser::Element`
#[proc_macro_derive(ParserElement, attributes(xml_data))]
pub fn derive_parser_element(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let meta = element::Meta::parse_meta(&input, true);

	let output = parser::build_parser(&meta, true);

	TokenStream::from(output)
}

/// Derive `xml-data::{parser,serializer}::Inner`
#[proc_macro_derive(Inner, attributes(xml_data))]
pub fn derive_inner(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let meta = element::Meta::parse_meta(&input, false);

	let mut output = serialize::build_serialize(&meta, false);
	output.extend(parser::build_parser(&meta, false));

	TokenStream::from(output)
}

/// Derive `xml-data::serializer::Inner`
#[proc_macro_derive(SerializerInner, attributes(xml_data))]
pub fn derive_serializer_inner(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let meta = element::Meta::parse_meta(&input, false);

	let output = serialize::build_serialize(&meta, false);

	TokenStream::from(output)
}

/// Derive `xml-data::parser::Inner`
#[proc_macro_derive(ParserInner, attributes(xml_data))]
pub fn derive_parser_inner(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let meta = element::Meta::parse_meta(&input, false);

	let output = parser::build_parser(&meta, false);

	TokenStream::from(output)
}
