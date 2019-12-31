use crate::{
	attributes::{
		all_attributes,
		string_lit,
		single_nested,
	},
};

use proc_macro2::Span;
use syn::{
	Data,
	DataStruct,
	DeriveInput,
	Fields,
	Ident,
	Lit,
	NestedMeta,
	Path,
	spanned::Spanned,
	Type,
	parse_quote,
};

pub struct FieldAttribute {
	pub key: String,
	pub optional: bool,
	pub is_string: bool,
}

pub struct Field {
	pub name: Ident,
	pub ty: Type,
	pub span: Span,
	pub attr: Option<FieldAttribute>,
}

impl Field {
	fn parse(name: Ident, field: &syn::Field) -> Self {
		let mut is_attr = false;
		let mut attr_key = None;
		let attr_optional = false;
		let mut attr_is_string = if let Type::Path(p) = &field.ty {
			p.qself.is_none() && p.path.is_ident("String")
		} else {
			false
		};

		for attr in all_attributes(&field.attrs) {
			let m = match attr {
				NestedMeta::Lit(_) => panic!("invalid literal in #[xml_data(..., ...)]"),
				NestedMeta::Meta(m) => m,
			};
			if m.path().is_ident("attr") {
				is_attr = true;
				let new_attr_key = string_lit(&m);
				if new_attr_key.is_some() {
					assert!(attr_key.is_none(), "Already have #[xml_data(attr(\"...\"))]");
				}
				attr_key = new_attr_key
			} else if m.path().is_ident("attr_string") {
				is_attr = true;
				attr_is_string = true;
			} else {
				panic!("Unknown #[xml_data] attribute");
			}
		}

		let attr = if is_attr {
			Some(FieldAttribute {
				key: attr_key.unwrap_or_else(|| name.to_string()),
				optional: attr_optional,
				is_string: attr_is_string,
			})
		} else { None };

		Field {
			name,
			span: field.span(),
			ty: field.ty.clone(),
			attr,
		}
	}
}

pub struct Meta {
	pub xml_data_crate: Path,
	pub name: Ident,
	pub tag: String,
	pub fields: Vec<Field>,
	pub ignore_unknown_attribute: bool,
	pub ignore_unknown_element: bool,
	pub ignore_text: bool,
}

impl Meta {
	pub fn parse_meta(element: &DeriveInput, impl_element: bool) -> Self {
		let mut tag = None;
		let mut xml_data_crate = None;
		let mut ignore_unknown_attribute = false;
		let mut ignore_unknown_element = false;
		let mut ignore_text = false;

		for attr in all_attributes(&element.attrs) {
			let m = match attr {
				NestedMeta::Lit(Lit::Str(t)) => {
					assert!(impl_element, "Tag not supported for `Inner`");
					assert!(tag.is_none(), "Already have #[xml_data(tag)]");
					tag = Some(t.value());
					continue;
				}
				NestedMeta::Lit(_) => panic!("invalid literal in #[xml_data(..., ...)]"),
				NestedMeta::Meta(m) => m,
			};
			if m.path().is_ident("tag") {
				assert!(impl_element, "Tag not supported for `Inner`");
				assert!(tag.is_none(), "Already have #[xml_data(tag)]");
				tag = Some("tag".into())
			} else if m.path().is_ident("crate") {
				assert!(xml_data_crate.is_none(), "Already have #[xml_data(crate)]");
				if let Some(NestedMeta::Meta(syn::Meta::Path(p))) = single_nested(&m) {
					xml_data_crate = Some(p.clone());
				} else {
					panic!("expected #[xml_data(crate(...))]");
				}
			} else if m.path().is_ident("ignore_unknown") {
				assert!(impl_element, "`ignore_unknown` not useful for `Inner`");
				ignore_unknown_attribute = true;
				ignore_unknown_element = true;
				ignore_text = true;
			} else {
				panic!("unknown #[xml_data()] attribute");
			}
		}
		// tag is ignored for `Inner`
		let tag = tag.unwrap_or_else(|| element.ident.to_string());
		let xml_data_crate = xml_data_crate.unwrap_or_else(|| parse_quote!{ xml_data });

		let fields = match &element.data {
			Data::Struct(DataStruct { fields: Fields::Named(n), .. } ) => {
				n.named.iter().map(|field| {
					Field::parse(field.ident.clone().unwrap(), &field)
				}).collect()
			},
			Data::Struct(DataStruct { fields: Fields::Unit, .. } ) => {
				// luckily unit structs now accept the `... {}` construction too.
				Vec::new()
			},
			_ => panic!("derive not supported on this type"),
		};

		Self {
			xml_data_crate,
			name: element.ident.clone(),
			tag,
			fields,
			ignore_unknown_attribute,
			ignore_unknown_element,
			ignore_text,
		}
	}
}
