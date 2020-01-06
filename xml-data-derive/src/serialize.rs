use crate::element::{
	Field,
	FieldAttribute,
	Meta,
};
use proc_macro2::TokenStream;
use quote::{
	quote,
	quote_spanned,
};

pub fn build_serialize(meta: &Meta, impl_element: bool) -> TokenStream {
	let Meta {
		xml_data_crate,
		name,
		tag,
		..
	} = meta;

	let serializer = if impl_element {
		quote! { &mut serializer }
	} else {
		quote! { serializer }
	};

	let el_attrs: TokenStream = meta
		.fields
		.iter()
		.filter_map(|field| {
			if let Some(attr) = &field.attr {
				if !impl_element {
					panic!("attributes now allowed in `Inner` data");
				}
				let FieldAttribute { key: attr_key, .. } = attr;
				let Field { name, span, .. } = field;
				let value_t = if attr.is_string {
					quote!(ValueString)
				} else {
					quote!(ValueDefault)
				};
				Some(if attr.optional {
					quote_spanned! {*span=>
						if let Some(#name) = &self.#name {
							serializer.serialize_attribute(#attr_key, #value_t::serialize_value(#name)?)?;
						}
					}
				} else {
					quote_spanned! {*span=>
						serializer.serialize_attribute(#attr_key, #value_t::serialize_value(&self.#name)?)?;
					}
				})
			} else {
				None
			}
		})
		.collect();
	let el_inner: TokenStream = meta
		.fields
		.iter()
		.filter_map(|field| {
			if field.attr.is_none() {
				let Field { name, span, .. } = field;
				Some(quote_spanned! {*span=>
					self.#name.serialize_elements(#serializer)?;
				})
			} else {
				None
			}
		})
		.collect();

	let actual_impl = if impl_element {
		quote! {
			impl FixedElement for #name {
				const TAG: &'static str = #tag;

				fn serialize<S: Serializer>(&self, mut serializer: S) -> Result<()> {
					#el_attrs
					#el_inner
					Ok(())
				}
			}
		}
	} else {
		quote! {
			impl Inner for #name {
				fn serialize_elements<S: Serializer>(&self, serializer: &mut S) -> Result<()> {
					#el_inner
					Ok(())
				}
			}
		}
	};

	quote! {
		const _: () = {
			use #xml_data_crate::{
				serializer::{
					FixedElement,
					Inner,
					Serializer,
					Value,
					ValueString,
					ValueDefault,
				},
				Result,
			};

			#actual_impl
		};
	}
}
