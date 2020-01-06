use crate::{
	errors,
	parser::{
		ElementParser,
		ElementState,
	},
	Result,
};
use std::borrow::Cow;

/// Convenience trait to implement instead of `ElementState` if your element has a fixed tag.
pub trait FixedElementState: Default {
	/// Same as `ElementState::Output`
	type Output: Sized;

	/// Fixed tag
	const TAG: &'static str;

	/// Same as `ElementState::parse_element_attribute`
	fn parse_element_attribute(&mut self, key: &str, value: Cow<'_, str>) -> Result<()> {
		let _ = value;
		return Err(errors::unexpected_attribute(key));
	}

	/// Same as `ElementState::parse_element_inner_text`
	fn parse_element_inner_text(&mut self, text: Cow<'_, str>) -> Result<()> {
		if !text.trim().is_empty() {
			return Err(errors::unexpected_text());
		}
		Ok(())
	}

	/// Same as `ElementState::parse_element_inner_node`
	fn parse_element_inner_node<P: ElementParser>(&mut self, tag: &str, parser: P) -> Result<()> {
		let _ = parser;
		return Err(errors::unexpected_element(tag));
	}

	/// Same as `ElementState::parse_element_finish`
	fn parse_element_finish(self) -> Result<Self::Output>;
}

impl<E: FixedElementState> ElementState for E {
	type Output = <E as FixedElementState>::Output;

	fn parse_element_start(tag: &str) -> Option<Self> {
		if tag == Self::TAG {
			Some(E::default())
		} else {
			None
		}
	}

	fn parse_element_attribute(&mut self, key: &str, value: Cow<'_, str>) -> Result<()> {
		<E as FixedElementState>::parse_element_attribute(self, key, value)
	}

	fn parse_element_inner_text(&mut self, text: Cow<'_, str>) -> Result<()> {
		<E as FixedElementState>::parse_element_inner_text(self, text)
	}

	fn parse_element_inner_node<P: ElementParser>(&mut self, tag: &str, parser: P) -> Result<()> {
		<E as FixedElementState>::parse_element_inner_node(self, tag, parser)
	}

	fn parse_element_finish(self) -> Result<Self::Output> {
		<E as FixedElementState>::parse_element_finish(self)
	}

	fn parse_error_not_found<T>() -> Result<T> {
		Err(errors::missing_element(Self::TAG))
	}
}
