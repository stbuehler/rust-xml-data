use xml_data::serializer::Element;

#[derive(Element)]
pub struct Example {
	#[xml_data(attr)]
	attr1: Option<String>,
}

fn main() {}
