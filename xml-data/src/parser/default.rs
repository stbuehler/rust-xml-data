use crate::{
	parser::{
		ElementState,
		InnerState,
	},
};

/// Parsable element
///
/// This links the (default) state type used to parse this element.
pub trait Element: Sized {
	/// Parse state to use for this element
	type ParseState: ElementState<Output = Self>;
}

/// Type alias to find the default parse state for an `Element`
pub type ElementDefaultParseState<E> = <E as Element>::ParseState;

/// Parsable inner data (multiple elements)
///
/// This links the (default) state type used to parse this.
pub trait Inner: Sized {
	/// Parse state to use for this inner data
	type ParseState: InnerState<Output = Self>;
}

/// Type alias to find the default parse state for an `Inner`
pub type InnerDefaultParseState<I> = <I as Inner>::ParseState;
