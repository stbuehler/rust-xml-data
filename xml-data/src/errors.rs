#![allow(missing_docs)] // names should be good enough
//! Helper functions to generate common errors

use crate::Error;
use std::fmt;

pub(crate) enum ParseError {
	UnexpectedEof { msg: String },
	UnexpectedEnd,
	UnexpectedDecl,
	UnexpectedDocType,
	UnexpectedPI,
	UnexpectedText,
	UnexpectedElement { tag: String },
	UnexpectedAttribute { key: String },
	InnerElementNotParsed { tag: String },
	MissingElement { tag: String },
	MissingUnknownElement,
	MissingAttribute { key: String },
}

impl fmt::Debug for ParseError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::UnexpectedEof { msg } => write!(f, "unexpected eof: {}", msg),
			Self::UnexpectedEnd => write!(f, "Unexpected end tag"),
			Self::UnexpectedDecl => write!(f, "Unexpected decl <?xml ... ?>"),
			Self::UnexpectedDocType => write!(f, "Unexpected <!DOCTYPE ...>"),
			Self::UnexpectedPI => write!(f, "Unexpected processing instructions <?...?>"),
			Self::UnexpectedText => write!(f, "Unexpected (non-whitespace) text/CDATA"),
			Self::UnexpectedElement { tag } => write!(f, "Unexpected element: {}", tag),
			Self::UnexpectedAttribute { key } => write!(f, "Unexpected attribute: {}", key),
			Self::InnerElementNotParsed { tag } => {
				write!(f, "Inner element {:?} wasn't fully parsed", tag)
			},
			Self::MissingElement { tag } => write!(f, "Missing element {:?}", tag),
			Self::MissingUnknownElement => write!(f, "Missing element"),
			Self::MissingAttribute { key } => write!(f, "Missing attribute {:?}", key),
		}
	}
}

impl fmt::Display for ParseError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Debug::fmt(self, f)
	}
}

impl std::error::Error for ParseError {}

pub fn unexpected_eof(msg: &str) -> Error {
	ParseError::UnexpectedEof { msg: msg.into() }.into()
}

pub fn unexpected_end() -> Error {
	ParseError::UnexpectedEnd.into()
}

pub fn unexpected_decl() -> Error {
	ParseError::UnexpectedDecl.into()
}

pub fn unexpected_doctype() -> Error {
	ParseError::UnexpectedDocType.into()
}

pub fn unexpected_pi() -> Error {
	ParseError::UnexpectedPI.into()
}

pub fn unexpected_text() -> Error {
	ParseError::UnexpectedText.into()
}

pub fn unexpected_element(tag: &str) -> Error {
	ParseError::UnexpectedElement { tag: tag.into() }.into()
}

pub fn unexpected_attribute(key: &str) -> Error {
	ParseError::UnexpectedAttribute { key: key.into() }.into()
}

pub fn inner_element_not_parsed(tag: &str) -> Error {
	ParseError::InnerElementNotParsed { tag: tag.into() }.into()
}

pub fn missing_element(tag: &str) -> Error {
	ParseError::MissingElement { tag: tag.into() }.into()
}

pub fn missing_unknown_element() -> Error {
	ParseError::MissingUnknownElement.into()
}

pub fn missing_attribute(key: &str) -> Error {
	ParseError::MissingAttribute { key: key.into() }.into()
}
