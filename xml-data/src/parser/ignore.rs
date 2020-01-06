use crate::{
	parser::{
		ElementParser,
		ElementState,
	},
	Result,
};
use std::borrow::Cow;

/// Can be used as `ElementState` to ignore an element with all content (attributes and sub
/// elements and text)
pub struct IgnoreElement;

impl ElementState for IgnoreElement {
	type Output = ();

	fn parse_element_start(_tag: &str) -> Option<Self> {
		Some(Self)
	}

	fn parse_element_attribute(&mut self, _key: &str, _value: Cow<'_, str>) -> Result<()> {
		Ok(())
	}

	fn parse_element_inner_text(&mut self, _text: Cow<'_, str>) -> Result<()> {
		Ok(())
	}

	fn parse_element_inner_node<P: ElementParser>(&mut self, _tag: &str, parser: P) -> Result<()> {
		// no need to create a new `IgnoreElement` state, just reuse `self`
		parser.parse_element_state(self)
	}

	fn parse_element_finish(self) -> Result<Self::Output> {
		Ok(())
	}
}
