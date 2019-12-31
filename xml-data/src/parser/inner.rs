use crate::{
	parser::{
		Element,
		ElementParser,
		Inner,
		ElementState,
	},
	Result,
};
use std::borrow::Cow;

/// Result of `InnerState` parse methods to signal whether they successfully parsed the input or
/// another `InnerState` needs to take a shot.
#[derive(Debug)]
pub enum InnerParseResult<Input> {
	/// Parsed successfully
	Success,
	/// Need something else to take the input
	Next(Input),
}

/// State to parse multiple elements (on the same level)
pub trait InnerState: Default {
	/// Once fully parsed this is the resulting output type.
	type Output: Sized;

	/// Try parsing an element with the given tag
	///
	/// Should not fail if it doesn't recognize the tag; instead it needs to return the parser.
	fn parse_inner_node<P: ElementParser>(&mut self, tag: &str, parser: P) -> Result<InnerParseResult<P>> {
		let _ = tag;
		Ok(InnerParseResult::Next(parser))
	}

	/// Try parsing inner text
	///
	/// Should not fail if it doesn't take text (but may fail if it does but can't parse it).
	fn parse_inner_text<'t>(&mut self, text: Cow<'t, str>) -> Result<InnerParseResult<Cow<'t, str>>> {
		Ok(InnerParseResult::Next(text))
	}

	/// Finish parsing.
	fn parse_inner_finish(self) -> Result<Self::Output>;
}

/// Using `String` as `InnerState` to collect all inner text in it (including whitespace in input)
impl InnerState for String {
	type Output = Self;

	fn parse_inner_text<'t>(&mut self, text: Cow<'t, str>) -> Result<InnerParseResult<Cow<'t, str>>> {
		if self.is_empty() {
			*self = text.into_owned();
		} else {
			*self += &text;
		}
		Ok(InnerParseResult::Success)
	}

	fn parse_inner_finish(self) -> Result<Self::Output> {
		Ok(self)
	}
}

impl Inner for String {
	type ParseState = String;
}

impl InnerState for Cow<'_, str> {
	type Output = Self;

	fn parse_inner_text<'t>(&mut self, text: Cow<'t, str>) -> Result<InnerParseResult<Cow<'t, str>>> {
		if self.is_empty() {
			*self = Cow::Owned(text.into_owned());
		} else {
			match self {
				Cow::Borrowed(s) => {
					let t = String::from(*s) + &text;
					*self = Cow::Owned(t);
				},
				Cow::Owned(s) => {
					*s += &text;
				},
			}
		}
		Ok(InnerParseResult::Success)
	}

	fn parse_inner_finish(self) -> Result<Self::Output> {
		Ok(self)
	}
}

/// `InnerState` to parse a single element
pub struct ParseElementOnce<E: ElementState> {
	element: Option<E::Output>,
}

impl<E: ElementState> Default for ParseElementOnce<E> {
	fn default() -> Self {
		Self { element: None }
	}
}

impl<E: ElementState> InnerState for ParseElementOnce<E> {
	type Output = E::Output;

	fn parse_inner_node<P: ElementParser>(&mut self, tag: &str, parser: P) -> Result<InnerParseResult<P>> {
		if self.element.is_none() {
			if let Some(mut state) = E::parse_element_start(tag) {
				parser.parse_element_state(&mut state)?;
				self.element = Some(state.parse_element_finish()?);
				return Ok(InnerParseResult::Success)
			}
		}
		Ok(InnerParseResult::Next(parser))
	}

	fn parse_inner_finish(self) -> Result<Self::Output> {
		if let Some(o) = self.element {
			Ok(o)
		} else {
			E::parse_error_not_found()
		}
	}
}

impl<E: Element> Inner for E {
	type ParseState = ParseElementOnce<E::ParseState>;
}

/// `InnerState` to parse a single optional element
pub struct ParseElementOptional<E: ElementState> {
	element: Option<E::Output>,
}

impl<E: ElementState> Default for ParseElementOptional<E> {
	fn default() -> Self {
		Self { element: None }
	}
}

impl<E: ElementState> InnerState for ParseElementOptional<E> {
	type Output = Option<E::Output>;

	fn parse_inner_node<P: ElementParser>(&mut self, tag: &str, parser: P) -> Result<InnerParseResult<P>> {
		if self.element.is_none() {
			if let Some(mut state) = E::parse_element_start(tag) {
				parser.parse_element_state(&mut state)?;
				self.element = Some(state.parse_element_finish()?);
				return Ok(InnerParseResult::Success)
			}
		}
		Ok(InnerParseResult::Next(parser))
	}

	fn parse_inner_finish(self) -> Result<Self::Output> {
		Ok(self.element)
	}
}

impl<E: Element> Inner for Option<E> {
	type ParseState = ParseElementOptional<E::ParseState>;
}

/// `InnerState` to parse multiple occurences of a single element
pub struct ParseElementList<E: ElementState> {
	elements: Vec<E::Output>,
}

impl<E: ElementState> Default for ParseElementList<E> {
	fn default() -> Self {
		Self {
			elements: Vec::new(),
		}
	}
}

impl<E: ElementState> InnerState for ParseElementList<E> {
	type Output = Vec<E::Output>;

	fn parse_inner_node<P: ElementParser>(&mut self, tag: &str, parser: P) -> Result<InnerParseResult<P>> {
		if let Some(mut state) = E::parse_element_start(tag) {
			parser.parse_element_state(&mut state)?;
			self.elements.push(state.parse_element_finish()?);
			Ok(InnerParseResult::Success)
		} else {
			Ok(InnerParseResult::Next(parser))
		}
	}

	fn parse_inner_finish(self) -> Result<Self::Output> {
		Ok(self.elements)
	}
}

impl<E: Element> Inner for Vec<E> {
	type ParseState = ParseElementList<E::ParseState>;
}

/// `InnerState` to parse optional inner data; if it parsed anything it needs to finish
pub struct ParseInnerOptional<I: InnerState> {
	inner: Option<I>,
}

impl<I: InnerState> Default for ParseInnerOptional<I> {
	fn default() -> Self {
		Self { inner: None }
	}
}

impl<I: InnerState> InnerState for ParseInnerOptional<I> {
	type Output = Option<I::Output>;

	fn parse_inner_node<P: ElementParser>(&mut self, tag: &str, parser: P) -> Result<InnerParseResult<P>> {
		if self.inner.is_none() {
			let mut state = I::default();
			match state.parse_inner_node(tag, parser)? {
				InnerParseResult::Success => (),
				InnerParseResult::Next(parser) => return Ok(InnerParseResult::Next(parser)),
			}
			// matched something successfully, remember state
			self.inner = Some(state);
			Ok(InnerParseResult::Success)
		} else if let Some(inner) = &mut self.inner {
			inner.parse_inner_node(tag, parser)
		} else {
			unreachable!()
		}
	}

	fn parse_inner_text<'t>(&mut self, text: Cow<'t, str>) -> Result<InnerParseResult<Cow<'t, str>>> {
		if self.inner.is_none() {
			let mut state = I::default();
			match state.parse_inner_text(text)? {
				InnerParseResult::Success => (),
				InnerParseResult::Next(text) => return Ok(InnerParseResult::Next(text)),
			}
			// matched something successfully, remember state
			self.inner = Some(state);
			Ok(InnerParseResult::Success)
		} else if let Some(inner) = &mut self.inner {
			inner.parse_inner_text(text)
		} else {
			unreachable!()
		}
	}

	fn parse_inner_finish(self) -> Result<Self::Output> {
		Ok(if let Some(inner) = self.inner {
			Some(inner.parse_inner_finish()?)
		} else {
			None
		})
	}
}
