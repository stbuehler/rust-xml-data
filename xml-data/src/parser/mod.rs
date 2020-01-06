//! Traits and helper structs to parse XML
//!
//! To implement parsing for your data types (mapping to XML elements) you need intermediate
//! "state" types (implementing `FixedElementState` or `ElementState`), which work like "builders":
//! they will receive the various parts incrementally until they can "build" the result.
//!
//! To define a default state for your type (so it can be easily found in certain places) you need
//! to implement `Element`.
//!
//! If your data type should represent multiple elements you need to a state type implementing
//! `InnerState`; the default state is defined by implementing `Inner`.  If `E` implements
//! `Element`, `E`, `Option<E>`, and `Vec<E>` automatically implement `Inner`.
//!
//! To implement parser adaptors for an XML library you need to implement `ElementParser`.

mod core;
mod default;
mod extensions;
mod fixed_element;
mod ignore;
mod inner;
mod value;

#[cfg(feature = "derive")]
pub use xml_data_derive::{
	ParserElement as Element,
	ParserInner as Inner,
};

pub use self::{
	core::{
		ElementParser,
		ElementState,
	},
	default::{
		Element,
		ElementDefaultParseState,
		Inner,
		InnerDefaultParseState,
	},
	extensions::ElementParserExt,
	fixed_element::FixedElementState,
	ignore::IgnoreElement,
	inner::{
		InnerParseResult,
		InnerState,
		ParseElementList,
		ParseElementOnce,
		ParseElementOptional,
		ParseInnerOptional,
	},
	value::{
		Value,
		ValueDefault,
		ValueString,
	},
};
