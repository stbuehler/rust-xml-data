use crate::{
	serializer::{
		self,
		Element,
	},
	Result,
};
use quick_xml::events::{
	attributes::Attribute,
	BytesDecl,
	BytesEnd,
	BytesStart,
	BytesText,
	Event,
};
use std::{
	borrow::Cow,
	io,
};

fn cow_bytes<'a>(value: Cow<'a, str>) -> Cow<'a, [u8]> {
	match value {
		Cow::Owned(v) => Cow::Owned(v.into()),
		Cow::Borrowed(v) => Cow::Borrowed(v.as_bytes()),
	}
}

/// Serialize element into full document in memory
pub fn serialize_document<E: Element>(element: &E) -> Result<String> {
	let mut buf = Vec::new();
	let mut writer = quick_xml::Writer::new(&mut buf);
	let mut serializer = Serializer::new(&mut writer);
	serializer.serialize_document(element)?;
	// there shouldn't be any way to write binary (non-utf8) data to the buffer
	Ok(String::from_utf8(buf).expect("buffer should be utf-8 clean"))
}

/// Serializer adaptor for `quick_xml::Writer`
pub struct Serializer<'w, W: io::Write> {
	writer: &'w mut quick_xml::Writer<W>,
}

impl<'w, W: io::Write> Serializer<'w, W> {
	/// New adaptor using the writer
	pub fn new(writer: &'w mut quick_xml::Writer<W>) -> Self {
		Self { writer }
	}

	/// Serialize full document from root element
	pub fn serialize_document<E: Element>(&mut self, element: &E) -> Result<()> {
		self.writer
			.write_event(Event::Decl(BytesDecl::new(b"1.1", Some(b"utf-8"), None)))?;
		self.serialize_element(element)
	}

	/// Serialize single element
	pub fn serialize_element<E: Element>(&mut self, element: &E) -> Result<()> {
		let tag = cow_bytes(element.tag());
		let mut ser = SRef {
			serializer: self,
			end: Some(BytesEnd::owned(tag.to_vec())),
			start: Some(BytesStart::owned_name(tag)),
		};
		element.serialize(&mut ser)?;
		ser.close()?;
		Ok(())
	}
}

struct SRef<'a, 'w, W: io::Write> {
	serializer: &'a mut Serializer<'w, W>,
	start: Option<BytesStart<'static>>,
	end: Option<BytesEnd<'static>>,
}

impl<'a, 'w, W: io::Write> SRef<'a, 'w, W> {
	fn start(&mut self) -> Result<()> {
		if let Some(s) = self.start.take() {
			self.serializer.writer.write_event(Event::Start(s))?;
		} else {
			assert!(self.end.is_some(), "element already closed");
		}
		Ok(())
	}

	fn close(&mut self) -> Result<()> {
		if let Some(s) = self.start.take() {
			self.serializer.writer.write_event(Event::Empty(s))?;
			self.end = None;
		} else if let Some(e) = self.end.take() {
			self.serializer.writer.write_event(Event::End(e))?;
		}
		Ok(())
	}
}

impl<'a, 'w, W: io::Write> serializer::Serializer for &'_ mut SRef<'a, 'w, W> {
	fn serialize_attribute(&mut self, key: &str, value: Cow<'_, str>) -> Result<()> {
		let start = self.start.as_mut().expect("element already started");
		let key = key.as_bytes();
		let value = cow_bytes(value);
		start.push_attribute(Attribute { key, value });
		Ok(())
	}

	fn serialize_text(&mut self, text: Cow<'_, str>) -> Result<()> {
		self.start()?;
		self.serializer
			.writer
			.write_event(Event::Text(BytesText::from_plain_str(&text)))?;
		Ok(())
	}

	fn serialize_element<E: Element>(&mut self, element: &E) -> Result<()> {
		self.start()?;
		self.serializer.serialize_element(element)
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::test_struct::*;

	#[test]
	fn test() {
		assert_eq!(
			serialize_document(&Data::TEST_RESULT_1).unwrap(),
			Data::TEST_SERIALIZE_DOCUMENT_1,
		);
	}
}
