use crate::{
	errors,
	Result,
};
use std::borrow::Cow;

/// A state to parse exactly one element
///
/// The idea is that a parser will try different implementors of this to parse an element it finds;
/// the first implementor returning `Some(..)` on `parse_element_start` will be used to actually
/// parse it.
///
/// After a successful `parse_element_start` the parser needs to call `parse_element_attribute` for
/// all attributes on the element, then `parse_element_inner_text` and `parse_element_inner_node`
/// until the closing tag of the element is hit, upon which it needs to call `parse_element_finish`.
pub trait ElementState: Sized {
	/// Once fully parsed this is the resulting output type.
	type Output: Sized;

	/// Try creating state to parse an element with the passed `tag`.
	fn parse_element_start(tag: &str) -> Option<Self>;

	/// Parse attribute into state
	///
	/// The default implementation will fail with "unexpected attribute".
	fn parse_element_attribute(&mut self, key: &str, value: Cow<'_, str>) -> Result<()> {
		let _ = value;
		Err(errors::unexpected_attribute(key))
	}

	/// Parse text or CDATA into state.
	///
	/// The default implementation will ignore whitespace and fail otherwise.
	fn parse_element_inner_text(&mut self, text: Cow<'_, str>) -> Result<()> {
		if !text.trim().is_empty() {
			return Err(errors::unexpected_text());
		}
		Ok(())
	}

	/// Parse inner elements.
	///
	/// The default implementation will fail with "unexpected element".
	fn parse_element_inner_node<P: ElementParser>(&mut self, tag: &str, parser: P) -> Result<()> {
		let _ = parser;
		Err(errors::unexpected_element(tag))
	}

	/// Finish parsing an element.
	///
	/// This is where you make sure you got all required data (unpacking their types) and can
	/// optionally check data for consistency.
	fn parse_element_finish(self) -> Result<Self::Output>;

	/// In case `parse_element_start` didn't get to accept any element (either because it always
	/// returned `None` or there just wasn't enough data), a parser can use this to generate an
	/// error.
	///
	/// `ParseElementOnce` uses this.
	fn parse_error_not_found<T>() -> Result<T> {
		Err(errors::missing_unknown_element())
	}
}

/// A parser that is ready to parse exactly one element (and nested data).
pub trait ElementParser: Sized {
	/// Start parsing an element with the prepared state
	///
	/// A parser will call the various `ElementState` to parse the element.
	///
	/// Users of this method will create the state using `ElementState::parse_element_start` and
	/// produce the final result using `ElementState::parse_element_finish` after calling this
	/// method.
	fn parse_element_state<E: ElementState>(self, state: &mut E) -> Result<()>;
}
