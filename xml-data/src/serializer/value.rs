use crate::{
	Result,
};
use std::borrow::Cow;

/// Trait to serialize attributes and inner text
///
/// This is implemented my "marker" types to decide how to serialize a type (the same type can be
/// serialized differently depending on the marker type)
pub trait Value<T> {
	/// Serialize value to text
	fn serialize_value(data: &T) -> Result<Cow<'_, str>>;
}

/// Implements `Value` for all types implementing `std::fmt::Display`; this is a good default.
pub struct ValueDefault;

impl<T: std::fmt::Display> Value<T> for ValueDefault {
	fn serialize_value(data: &T) -> Result<Cow<'_, str>> {
		Ok(Cow::Owned(data.to_string()))
	}
}

/// Implements `Value` for all types implementing `AsRef<str>`.
pub struct ValueString;

impl<T: AsRef<str>> Value<T> for ValueString {
	fn serialize_value(data: &T) -> Result<Cow<'_, str>> {
		Ok(Cow::Borrowed(data.as_ref()))
	}
}
