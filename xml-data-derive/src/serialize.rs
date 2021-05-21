use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens, TokenStreamExt};
use syn::{spanned::Spanned, Path};

use crate::element::{
	self, ElementInput, FieldAttribute, FieldBase, FieldChild, InnerInput, SField,
	SerdeElementInput,
};

struct ElementChild<'a> {
	data: &'a FieldChild,
	xml_data_crate: &'a Path,
	serializer: &'a TokenStream,
}

impl<'a> ElementChild<'a> {
	fn new(data: &'a FieldChild, xml_data_crate: &'a Path, serializer: &'a TokenStream) -> Self {
		Self {
			data,
			xml_data_crate,
			serializer,
		}
	}
}

impl ToTokens for ElementChild<'_> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let ident = &self.data.ident;
		let Self {
			xml_data_crate,
			serializer,
			..
		} = self;

		tokens.append_all(quote_spanned! {self.data.span()=>
			#xml_data_crate::serializer::Inner::serialize_elements(&self.#ident, #serializer)?;
		});
	}
}

struct ElementAttribute<'a> {
	data: &'a FieldAttribute,
	xml_data_crate: &'a Path,
}

impl<'a> ElementAttribute<'a> {
	fn new(data: &'a FieldAttribute, xml_data_crate: &'a Path) -> Self {
		Self {
			data,
			xml_data_crate,
		}
	}
}

impl ToTokens for ElementAttribute<'_> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let Self {
			data,
			xml_data_crate,
		} = self;

		let value_t = if data.is_string {
			quote!(ValueString)
		} else {
			quote!(ValueDefault)
		};

		let ident = &data.ident;
		let attr_key = &data.key;

		if data.optional {
			tokens.append_all(quote_spanned! {self.data.span()=>
				if let Some(#ident) = &self.#ident {
					serializer.serialize_attribute(#attr_key, #xml_data_crate::serializer::#value_t::serialize_value(#ident)?)?;
				}
			});
		} else {
			tokens.append_all(quote_spanned! {self.data.span()=>
				serializer.serialize_attribute(#attr_key, #xml_data_crate::serializer::#value_t::serialize_value(&self.#ident)?)?;
			});
		}
	}
}

struct SElementAttribute<'a>(SField<'a, &'a FieldAttribute>);

impl ToTokens for SElementAttribute<'_> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let data = &self.0;

		if data.serde.attrs.skip_serializing() {
			return;
		}

		let xml_data_crate = data.xml_data_crate();
		let ident = data.ident();
		let attr_key = &data.xml.key;
		let value_t = if data.xml.is_string {
			quote!(ValueString)
		} else {
			quote!(ValueDefault)
		};

		let serialize_expr = if data.xml.optional {
			quote_spanned! {data.span()=>
				if let Some(#ident) = &self.#ident {
					serializer.serialize_attribute(#attr_key, #xml_data_crate::serializer::#value_t::serialize_value(#ident)?)?;
				}
			}
		} else {
			quote_spanned! {data.span()=>
				serializer.serialize_attribute(#attr_key, #xml_data_crate::serializer::#value_t::serialize_value(&self.#ident)?)?;
			}
		};

		if let Some(skip_serializing_if) = data.serde.attrs.skip_serializing_if() {
			tokens.append_all(quote! {
				if !#skip_serializing_if(&self.#ident) {
					#serialize_expr
				}
			});
		} else {
			tokens.append_all(serialize_expr);
		}
	}
}

struct SElementChild<'a> {
	data: SField<'a, &'a FieldChild>,
	serializer: &'a TokenStream,
}

impl ToTokens for SElementChild<'_> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let Self { data, serializer } = self;

		if data.serde.attrs.skip_serializing() {
			return;
		}

		let xml_data_crate = data.xml_data_crate();
		let ident = data.ident();

		let serialize_expr = quote_spanned! {data.span()=>
			#xml_data_crate::serializer::Inner::serialize_elements(&self.#ident, #serializer)?;
		};

		if let Some(skip_serializing_if) = data.serde.attrs.skip_serializing_if() {
			tokens.append_all(quote! {
				if !#skip_serializing_if(&self.#ident) {
					#serialize_expr
				}
			});
		} else {
			tokens.append_all(serialize_expr);
		}
	}
}

pub fn s_derive_fixed(el: &SerdeElementInput) -> TokenStream {
	let serializer = quote!(&mut serializer);
	let el_attrs = el.attrs().map(SElementAttribute).collect::<Vec<_>>();
	let el_body = el
		.children()
		.map(|data| SElementChild {
			data,
			serializer: &serializer,
		})
		.collect::<Vec<_>>();

	let xml_data_crate = &el.xml.xml_data_crate;
	let ident = &el.xml.ident;
	let tag = el.xml.tag();

	quote! {
		impl #xml_data_crate::serializer::FixedElement for #ident {
			const TAG: &'static str = #tag;

			fn serialize<S: #xml_data_crate::serializer::Serializer>(&self, mut serializer: S) -> #xml_data_crate::Result<()> {
				use #xml_data_crate::serializer::Value;
				#(#el_attrs)*
				#(#el_body)*
				Ok(())
			}
		}
	}
}

pub fn derive_fixed_element(el: &element::ElementInput) -> TokenStream {
	let serializer = quote!(&mut serializer);
	let xml_data_crate = &el.xml_data_crate;
	let el_attrs = el
		.attrs()
		.map(|field| ElementAttribute::new(field, xml_data_crate))
		.collect::<Vec<_>>();
	let el_body = el
		.elements()
		.map(|field| ElementChild::new(field, xml_data_crate, &serializer))
		.collect::<Vec<_>>();

	let ElementInput {
		xml_data_crate,
		ident,
		..
	} = el;

	let tag = el.tag();

	quote! {
		impl #xml_data_crate::serializer::FixedElement for #ident {
			const TAG: &'static str = #tag;

			fn serialize<S: #xml_data_crate::serializer::Serializer>(&self, mut serializer: S) -> #xml_data_crate::Result<()> {
				use #xml_data_crate::serializer::Value;
				#(#el_attrs)*
				#(#el_body)*
				Ok(())
			}
		}
	}
}

pub fn derive_inner(el: &InnerInput) -> TokenStream {
	let serializer = quote!(serializer);
	let xml_data_crate = &el.xml_data_crate;
	let children = el
		.elements()
		.map(|field| ElementChild::new(field, xml_data_crate, &serializer))
		.collect::<Vec<_>>();
	let InnerInput {
		ident,
		xml_data_crate,
		..
	} = el;

	quote! {
		impl #xml_data_crate::serializer::Inner for #ident {
			fn serialize_elements<S: #xml_data_crate::serializer::Serializer>(&self, serializer: &mut S) -> #xml_data_crate::Result<()> {
				#(#children)*
				Ok(())
			}
		}
	}
}
