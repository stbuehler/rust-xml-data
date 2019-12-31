use crate::{
	serializer::{
		Element,
		Serializer,
	},
	Result,
};
use std::borrow::Cow;

/// Convenience interface to serialize collections of elements
pub trait Inner {
	/// Serialize all contained elements
	fn serialize_elements<S: Serializer>(&self, serializer: &mut S) -> Result<()>;
}

/// Simply serialize inner text
impl Inner for String {
	fn serialize_elements<S: Serializer>(&self, serializer: &mut S) -> Result<()> {
		serializer.serialize_text(self.into())
	}
}

impl Inner for Cow<'_, str> {
	fn serialize_elements<S: Serializer>(&self, serializer: &mut S) -> Result<()> {
		serializer.serialize_text(self.as_ref().into())
	}
}

/// Simply serialize the element
impl<E: Element> Inner for E {
	fn serialize_elements<S: Serializer>(&self, serializer: &mut S) -> Result<()> {
		serializer.serialize_element(self)
	}
}

/// Serialize inner data if present
impl<I: Inner> Inner for Option<I> {
	fn serialize_elements<S: Serializer>(&self, serializer: &mut S) -> Result<()> {
		if let Some(i) = self {
			i.serialize_elements(serializer)?;
		}
		Ok(())
	}
}

/// Serialize all inner data
impl<I: Inner> Inner for Vec<I> {
	fn serialize_elements<S: Serializer>(&self, serializer: &mut S) -> Result<()> {
		for i in self {
			i.serialize_elements(serializer)?;
		}
		Ok(())
	}
}
