#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![warn(missing_docs)]
#![doc(html_root_url = "https://docs.rs/xml-data/0.0.1")]
//! This library provides a generic interface to parse XML data: a user might implement how to
//! parse and serialize their data (possibly derived), while others will implement adaptors for
//! generic XML parsers.
//!
//! This is similar to what serde does; but serde assumes your data consists of "native data"
//! (strings, integers, floats, ...) and nested data (lists and maps).  XML doesn't map to this
//! very well; while there are some adaptors, they often accept lots of structually different input
//! data: an element might be interpreted as map in serde. A subelement with text now can be
//! interpreted as key (`<key>value</key>`), but an attribute is interpreted the same way `<...
//! key="value">`.
//!
//! This library focuses only on XML instead, and provides a more strict interface with clearly
//! defined output.
//!
//! For the following XML handling crates adaptors are included if enabled through the equally
//! named features:
//! - [`quick-xml`](https://crates.io/crates/quick-xml)
//!
//! If the `derive` feature is enabled the following traits can be derived:
//! - `Element`
//! - `parser::Element`
//! - `serializer::Element`
//! - `Inner`
//! - `parser::Inner`
//! - `serializer::Inner`

pub mod errors;
pub mod extensions;
pub mod parser;
pub mod serializer;
mod traits;

/// For now we use a simple boxed error to show the user
pub type Error = Box<dyn std::error::Error>;
/// Result alias with out error type included
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(feature = "quick-xml")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "quick-xml")))]
pub mod quick_xml;

#[cfg(any(test, feature = "_private-test"))]
mod test_struct;

pub use self::traits::{
	Element,
	Inner,
};

#[cfg_attr(doc_cfg, doc(cfg(feature = "derive")))]
#[cfg(feature = "derive")]
pub use xml_data_derive::{
	Element,
	Inner,
};
