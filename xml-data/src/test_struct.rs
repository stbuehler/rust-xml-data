use std::borrow::Cow;

#[derive(crate::Element)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[xml_data("data", crate(crate))]
pub struct Data {
	#[xml_data(attr_string)]
	pub key: Cow<'static, str>,
	#[xml_data(attr)]
	pub other: u32,
	pub foo1: Option<Foo>,
	pub inner: DataInner,
}

#[derive(crate::Inner)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[xml_data(crate(crate))]
pub struct DataInner {
	pub foo2: Option<Foo>,
	pub content: String,
}

impl Data {
	pub const TEST_PARSE_DOCUMENT_1: &'static str = r#"<?xml version="1.1" encoding="utf-8"?>
<data key="abc" other="42"><foo>
		<unknown/>
	</foo></data>"#;

	pub const TEST_SERIALIZE_DOCUMENT_1: &'static str = r#"<?xml version="1.1" encoding="utf-8"?><data key="abc" other="42"><foo/></data>"#;

	pub const TEST_RESULT_1: Self = Self {
		key: Cow::Borrowed("abc"),
		other: 42,
		foo1: Some(Foo),
		inner: DataInner {
			foo2: None,
			content: String::new(),
		},
	};
}

#[derive(crate::Element)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[xml_data("foo", crate(crate), ignore_unknown)]
pub struct Foo;
