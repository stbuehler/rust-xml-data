#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2018::*;
#[macro_use]
extern crate std;
use xml_data::serializer::Element;
pub struct Example {
    #[xml_data(attr(optional))]
    attr1: Option<String>,
}
impl xml_data::serializer::FixedElement for Example {
    const TAG: &'static str = "Example";
    fn serialize<S: xml_data::serializer::Serializer>(
        &self,
        mut serializer: S,
    ) -> xml_data::Result<()> {
        use xml_data::serializer::Value;
        if let Some(attr1) = &self.attr1 {
            serializer.serialize_attribute(
                "attr1",
                xml_data::serializer::ValueDefault::serialize_value(attr1)?,
            )?;
        }
        Ok(())
    }
}
fn main() {}
