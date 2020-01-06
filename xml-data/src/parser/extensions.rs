use crate::{
	errors,
	parser::{
		ElementParser,
		ElementState,
	},
	Result,
};

/// extend `ElementParser` trait with convenience methods
pub trait ElementParserExt: ElementParser {
	/// Full parsing of an element (fails hard if the tag doesn't work out)
	///
	/// If you need to handle `parse_element_start` failures (e.g. by trying a different state) you
	/// can't use this method.
	fn parse_element<E: ElementState>(self, tag: &str) -> Result<E::Output> {
		let mut state = match E::parse_element_start(tag) {
			Some(s) => s,
			None => return Err(errors::unexpected_element(tag)),
		};
		self.parse_element_state(&mut state)?;
		state.parse_element_finish()
	}
}

impl<P: ElementParser> ElementParserExt for P {}
