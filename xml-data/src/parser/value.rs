use crate::Result;
use std::borrow::Cow;

/// Trait to parse attributes and inner text
///
/// This is implemented my "marker" types to decide how to parse a type (the same type can be
/// parsed differently depending on the marker type)
pub trait Value<T> {
	/// Serialize value to text
	fn parse_value(text: Cow<'_, str>) -> Result<T>;
}

/// Implements `Value` for all types implementing `std::str::FromStr`; this is a good default.
pub struct ValueDefault;

impl<T> Value<T> for ValueDefault
where
	T: std::str::FromStr,
	T::Err: std::error::Error + 'static,
{
	fn parse_value(text: Cow<'_, str>) -> Result<T> {
		Ok(text.parse::<T>()?)
	}
}

/// Implements `Value` for `String` and `Cow<str>`.
pub struct ValueString;

impl Value<String> for ValueString {
	fn parse_value(text: Cow<'_, str>) -> Result<String> {
		Ok(text.into_owned())
	}
}

impl<'a> Value<Cow<'a, str>> for ValueString {
	fn parse_value(text: Cow<'_, str>) -> Result<Cow<'a, str>> {
		Ok(Cow::Owned(text.into_owned()))
	}
}
