// use `cargo run -p xml-data --example struct_custom_tag` to run from git repository base directory

use std::borrow::Cow;
use xml_data::{quick_xml::serialize_document, Element};

/// a struct that customizes the tag name in XML serialization
#[derive(Element)]
#[xml_data(tag = "datum")]
pub struct Data {
	#[xml_data(attr_string)]
	pub key: Cow<'static, str>,
	#[xml_data(attr)]
	pub other: u32,
}

fn main() {
	let stuff = Data {
		key: Cow::from("hello"),
		other: 5,
	};

	eprintln!("Generating XML output...");

	// print serialized document
	println!("{}", serialize_document(&stuff).unwrap());
}
