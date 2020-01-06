use crate::Result;
use std::borrow::Cow;

/// Element that can be serialized.
pub trait Element {
	/// Tag for XML element
	fn tag(&self) -> Cow<'_, str>;

	/// Called by serializer to let an element serialize its attributes and inner data (text and
	/// further elements).
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<()>;
}

/// Interface to serialize an element.
///
/// An element needs to serialize attributes first, then inner text and elements.
pub trait Serializer {
	/// Add an attribute to the serialized element
	fn serialize_attribute(&mut self, key: &str, value: Cow<'_, str>) -> Result<()>;

	/// Add inner text to the element.
	///
	/// Must be escaped automatically by the serializer.
	fn serialize_text(&mut self, text: Cow<'_, str>) -> Result<()>;

	/// Add an inner element
	///
	/// The serializer will need to determine the `Element::tag` of the element and call its
	/// `Element::serialize` function.
	fn serialize_element<E: Element>(&mut self, element: &E) -> Result<()>;
}
