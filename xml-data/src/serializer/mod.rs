//! Traits and helper structs to serialize XML
//!
//! To implement serializing for your data types (mapping to XML elements) you need to implement
//! `FixedElement` or `Element`.
//!
//! If your data type represents multiple elements you need to implement `Inner`.
//!
//! To implement serialize adaptors for an XML library you need to implement `Serializer`.

mod core;
mod fixed_element;
mod inner;
mod value;

#[cfg(feature = "derive")]
pub use xml_data_derive::{
	ParserInner as Inner,
	SerializerElement as Element,
};

pub use self::{
	core::{
		Element,
		Serializer,
	},
	fixed_element::FixedElement,
	inner::Inner,
	value::{
		Value,
		ValueDefault,
		ValueString,
	},
};
