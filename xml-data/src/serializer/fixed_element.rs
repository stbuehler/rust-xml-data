use crate::{
	serializer::{
		Element,
		Serializer,
	},
	Result,
};
use std::borrow::Cow;

/// Serializable element with a fixed tag.
pub trait FixedElement {
	/// Fixed tag
	const TAG: &'static str;

	/// Same as `Element::serialize`.
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<()>;
}

impl<E: FixedElement> Element for E {
	fn tag(&self) -> Cow<'_, str> {
		Cow::Borrowed(Self::TAG)
	}

	fn serialize<S: Serializer>(&self, serializer: S) -> Result<()> {
		<Self as FixedElement>::serialize(self, serializer)
	}
}
