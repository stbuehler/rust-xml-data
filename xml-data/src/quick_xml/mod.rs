//! Parser/serializer adaptors using `quick-xml`

mod parser;
mod serializer;

/// Re-export `quick-xml` crate
pub use quick_xml;

pub use self::{
	parser::Parser,
	serializer::{
		serialize_document,
		Serializer,
	},
};
