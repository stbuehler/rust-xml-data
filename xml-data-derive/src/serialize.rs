use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens, TokenStreamExt};
use syn::spanned::Spanned;

use crate::element::{
	CrateRoot, ElementInput, FieldAttribute, FieldBase, FieldChild, InnerInput, SField,
};

struct ElementAttribute<'a, P>(SField<'a, &'a FieldAttribute, P>);

impl<P: CrateRoot> ToTokens for ElementAttribute<'_, P> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let data = &self.0;

		if data.serde.attrs.skip_serializing() {
			return;
		}

		let xml_data_crate = data.crate_root();
		let ident = data.ident();
		let attr_key = data.name().serialize_name();
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

struct ElementChild<'a, P> {
	data: SField<'a, &'a FieldChild, P>,
	serializer: &'a TokenStream,
}

impl<P: CrateRoot> ToTokens for ElementChild<'_, P> {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let Self { data, serializer } = self;

		if data.serde.attrs.skip_serializing() {
			return;
		}

		let xml_data_crate = data.crate_root();
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

pub fn derive_element(el: &ElementInput) -> TokenStream {
	let serializer = quote!(&mut serializer);
	let el_attrs = el.attrs().map(ElementAttribute).collect::<Vec<_>>();
	let el_body = el
		.children()
		.map(|data| ElementChild {
			data,
			serializer: &serializer,
		})
		.collect::<Vec<_>>();

	let xml_data_crate = el.crate_root();
	let ident = el.ident();
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
	let crate_root = el.crate_root();
	let ident = el.ident();
	let children = el
		.children()
		.map(|data| ElementChild {
			data,
			serializer: &serializer,
		})
		.collect::<Vec<_>>();

	quote! {
		impl #crate_root::serializer::Inner for #ident {
			fn serialize_elements<S: #crate_root::serializer::Serializer>(&self, serializer: &mut S) -> #crate_root::Result<()> {
				#(#children)*
				Ok(())
			}
		}
	}
}
