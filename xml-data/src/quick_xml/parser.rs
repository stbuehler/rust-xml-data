use crate::{
	extensions::*,
	parser::{
		ElementParser,
		ElementState,
		Element,
	},
	Result,
	errors,
};
use quick_xml::events::Event;
use std::io::BufRead;

/// Parser adaptor for `quick_xml::Reader`
pub struct Parser<'a, 'r, B: BufRead> {
	inner: &'r mut quick_xml::Reader<B>,
	buf: &'a mut Vec<u8>, // buf is used for pending; must not be touched if pending isn't None.
	pending: Option<Event<'a>>,
}

impl<'a, 'r, B: BufRead> Parser<'a, 'r, B> {
	/// Create a new adaptor from a reader and a buffer
	pub fn new(inner: &'r mut quick_xml::Reader<B>, buf: &'a mut Vec<u8>) -> Self {
		Self {
			inner,
			buf,
			pending: None,
		}
	}

	fn shift(&mut self) -> Result<()> {
		self.pending = None; // release buf reference
		let buf: &'static mut Vec<u8> = unsafe { &mut *(self.buf as *mut _) };
		self.pending = Some(self.inner.read_event(buf)?);
		Ok(())
	}

	fn peek(&mut self) -> Result<Event<'a>> {
		if self.pending.is_none() { self.shift()?; }
		Ok(self.pending.clone().expect("can't be None"))
	}

	fn clear(&mut self) {
		self.pending = None;
	}

	/// Parse a single (root) element from reading a document
	///
	/// Uses the default state type for the returned element.
	pub fn parse_document<E: Element>(&mut self) -> Result<E> {
		self.parse_document_for_state::<E::ParseState>()
	}

	/// Parse a single (root) element from reading a document
	///
	/// Uses the given state type.
	pub fn parse_document_for_state<S: ElementState>(&mut self) -> Result<S::Output> {
		let mut output = None;
		loop {
			match self.peek()? {
				Event::Eof => {
					if let Some(o) = output {
						return Ok(o);
					}
					return Err(errors::unexpected_eof("empty document"));
				},
				Event::End(_) => {
					return Err(errors::unexpected_end());
				},
				Event::Start(s)|Event::Empty(s) => {
					let tag = self.inner.decode(s.name());
					let mut finished_inner = false;
					let p = PRef { parser: self, finished_element: &mut finished_inner };
					output = Some(p.parse_element::<S>(&tag)?);
					if !finished_inner {
						return Err(errors::inner_element_not_parsed(&tag));
					}
					continue;
				},
				// not supported
				Event::PI(_) => return Err(errors::unexpected_pi()),
				// ignore those at document level before the root element
				Event::Decl(_) => {
					if output.is_some() {
						return Err(errors::unexpected_decl());
					}
				},
				Event::DocType(_) => {
					if output.is_some() {
						return Err(errors::unexpected_doctype());
					}
				},
				// ignore comments
				Event::Comment(_) => (),
				// text+cdata
				Event::Text(t)|Event::CData(t) => {
					let t = t.unescape_and_decode(self.inner)?;
					if !t.trim().is_empty() {
						return Err(errors::unexpected_text());
					}
				},
			}
			// Start+Empty continue directly; everything else needs to be cleared so we don't read it again
			self.clear();
		}
	}
}

struct PRef<'x, 'a, 'r, B: BufRead> {
	parser: &'x mut Parser<'a, 'r, B>,
	finished_element: &'x mut bool,
}

impl<'x, 'a, 'r, B: BufRead> ElementParser for PRef<'x, 'a, 'r, B> {
	fn parse_element_state<E: ElementState>(self, state: &mut E) -> Result<()> {
		let (start, closed) = match self.parser.peek()? {
			Event::Start(s) => (s, false),
			Event::Empty(s) => (s, true),
			_ => panic!("Element::read requires start or empty event"),
		};

		// TODO: quick-xml decoding sucks. no proper handling, "encoding" feature breaks API.
		// improve quick-xml, then use it here

		for attr in start.attributes() {
			let attr = attr?;
			let attr_key = self.parser.inner.decode(attr.key);
			let attr_value = attr.unescape_and_decode_value(self.parser.inner)?;
			state.parse_element_attribute(&attr_key, attr_value.into())?;
		}

		self.parser.clear(); // consume start tag

		// read inner (unless there is no inner)
		if closed {
			*self.finished_element = true;
			return Ok(());
		}

		loop {
			match self.parser.peek()? {
				Event::Eof => return Err(errors::unexpected_eof("unclosed element")),
				Event::End(_) => {
					self.parser.clear();
					*self.finished_element = true;
					return Ok(());
				},
				Event::Start(s)|Event::Empty(s) => {
					let tag = self.parser.inner.decode(s.name());
					let mut finished_inner = false;
					let p = PRef { parser: self.parser, finished_element: &mut finished_inner };
					state.parse_element_inner_node(&tag, p)?;
					if !finished_inner {
						return Err(errors::inner_element_not_parsed(&tag));
					}
					continue;
				},
				// not supported
				Event::PI(_) => return Err(errors::unexpected_pi()),
				// within elements those shouldn't be there
				Event::Decl(_) => return Err(errors::unexpected_decl()),
				Event::DocType(_) => return Err(errors::unexpected_doctype()),
				// ignore comments
				Event::Comment(_) => (),
				// text+cdata
				Event::Text(t)|Event::CData(t) => {
					let t = t.unescape_and_decode(self.parser.inner)?;
					state.parse_element_inner_text(t.into())?;
				},
			}
			// Start+Empty continue directly; everything else needs to be cleared so we don't read it again
			self.parser.clear();
		}
	}
}

#[cfg(test)]
mod test {
	use crate::Result;
	use crate::test_struct::*;

	fn parse<T: super::Element>(input: &str) -> Result<T> {
		let mut r = quick_xml::Reader::from_reader(std::io::Cursor::new(input));
		let mut buf = Vec::new();
		let mut p = super::Parser::new(&mut r, &mut buf);
		p.parse_document::<T>()
	}

	#[test]
	fn test() {
		assert_eq!(
			parse::<Data>(Data::TEST_PARSE_DOCUMENT_1).unwrap(),
			Data::TEST_RESULT_1,
		);
	}
}
