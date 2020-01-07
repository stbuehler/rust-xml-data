use crate::{
	parser,
	serializer,
};

/// Combining [`parser::Element`] and [`serializer::Element`].
///
/// Can be derived (if `derive` feature is active).
pub trait Element: parser::Element + serializer::Element {}

impl<E: parser::Element + serializer::Element> Element for E {}

/// Combining [`parser::Inner`] and [`serializer::Inner`].
///
/// Can be derived (if `derive` feature is active).
pub trait Inner: parser::Inner + serializer::Inner {}

impl<E: parser::Inner + serializer::Inner> Inner for E {}
