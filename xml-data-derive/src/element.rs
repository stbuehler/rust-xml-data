use std::{borrow::Cow, ops::Deref};

use darling::{
	ast,
	util::{Flag, Override, SpannedValue},
	FromDeriveInput, FromField, FromMeta,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, Ident, Path, Type};

/// Parsed representation of a field that should be expressed as an XML attribute.
pub struct FieldAttribute {
	pub ident: Ident,
	pub ty: Type,
	pub key: String,
	pub optional: bool,
	pub is_string: bool,
}

impl FieldAttribute {
	/// Get the "value type" used for serialization and deserialization.
	///
	/// Despite having the same `Ident`, these are actually distinct types in different modules.
	pub(crate) fn value_type(&self) -> TokenStream {
		if self.is_string {
			quote!(ValueString)
		} else {
			quote!(ValueDefault)
		}
	}
}

pub struct FieldChild {
	pub ident: Ident,
	pub ty: Type,
}

impl FromField for FieldChild {
	fn from_field(field: &syn::Field) -> darling::Result<Self> {
		if let Some(ident) = field.ident.clone() {
			Ok(Self {
				ident,
				ty: field.ty.clone(),
			})
		} else {
			Err(darling::Error::custom("Only named fields supported"))
		}
	}
}

/// A field on the deriving struct. Fields can be expressed in XML as either attributes
/// or child elements.
pub enum Field {
	Attribute(FieldAttribute),
	Child(FieldChild),
}

impl FromField for Field {
	fn from_field(field: &syn::Field) -> darling::Result<Self> {
		#[derive(Default, FromMeta)]
		#[darling(default)]
		struct RawFieldAttr {
			pub key: Option<String>,
			pub optional: bool,
		}

		#[derive(FromField)]
		#[darling(attributes(xml_data))]
		struct RawField {
			ident: Option<Ident>,
			ty: Type,
			#[darling(default)]
			attr: Option<Override<RawFieldAttr>>,
			#[darling(default)]
			attr_string: Flag,
		}

		let RawField {
			ident,
			ty,
			attr,
			attr_string,
		} = RawField::from_field(field)?;

		if attr.is_some() && attr_string.is_some() {
			todo!()
		}

		let ident = ident.expect("Only named structs are supported");

		if let Some(attr) = attr {
			let attr = attr.unwrap_or_default();
			Ok(Field::Attribute(FieldAttribute {
				key: attr.key.unwrap_or_else(|| ident.to_string()),
				is_string: if let Type::Path(pt) = &ty {
					pt.qself.is_none() && pt.path.is_ident("String")
				} else {
					false
				},
				optional: attr.optional,
				ident,
				ty,
			}))
		} else if attr_string.is_some() {
			Ok(Field::Attribute(FieldAttribute {
				key: ident.to_string(),
				is_string: true,
				optional: false,
				ident,
				ty,
			}))
		} else {
			Ok(Field::Child(FieldChild { ident, ty }))
		}
	}
}

#[derive(Default, FromMeta)]
pub struct IgnoreUnknown(bool);

impl IgnoreUnknown {
	pub fn elements(&self) -> bool {
		self.0
	}

	pub fn attributes(&self) -> bool {
		self.0
	}

	pub fn text(&self) -> bool {
		self.0
	}
}

fn default_crate_path() -> Path {
	parse_quote!(xml_data)
}

#[derive(FromDeriveInput)]
#[darling(attributes(xml_data), supports(struct_named, struct_unit))]
pub struct ElementInput {
	pub ident: Ident,
	pub data: ast::Data<(), SpannedValue<Field>>,
	/// If set, the XML tag name to use instead of the deriving struct ident.
	#[darling(default)]
	tag: Option<String>,
	#[darling(rename = "crate", default = "default_crate_path")]
	pub xml_data_crate: Path,
	#[darling(default)]
	pub ignore_unknown: IgnoreUnknown,
}

impl ElementInput {
	/// The XML tag name for the element during serialization and deserialization.
	pub fn tag(&self) -> Cow<'_, str> {
		if let Some(explicit_tag) = &self.tag {
			Cow::Borrowed(explicit_tag)
		} else {
			Cow::Owned(self.ident.to_string())
		}
	}

	/// The fields of the input struct.
	pub fn fields<'a>(&'a self) -> impl Iterator<Item = &'a SpannedValue<Field>> {
		self.data.as_ref().take_struct().unwrap().into_iter()
	}

	/// Fields of the input struct that are represented as attributes.
	pub fn attrs<'a>(&'a self) -> impl Iterator<Item = SpannedValue<&'a FieldAttribute>> {
		self.fields().filter_map(|field| {
			let span = field.span();
			if let Field::Attribute(attr) = field.deref() {
				Some(SpannedValue::new(attr, span))
			} else {
				None
			}
		})
	}

	/// Fields of the input struct that are represented as child elements.
	pub fn elements<'a>(&'a self) -> impl Iterator<Item = SpannedValue<&'a FieldChild>> {
		self.fields().filter_map(|field| {
			let span = field.span();
			if let Field::Child(child) = field.deref() {
				Some(SpannedValue::new(child, span))
			} else {
				None
			}
		})
	}
}

#[derive(FromDeriveInput)]
#[darling(attributes(xml_data), supports(struct_named, struct_unit))]
pub struct InnerInput {
	pub ident: Ident,
	pub data: ast::Data<(), SpannedValue<FieldChild>>,
	#[darling(rename = "crate", default = "default_crate_path")]
	pub xml_data_crate: Path,
}

impl InnerInput {
	pub fn elements<'a>(&'a self) -> impl Iterator<Item = SpannedValue<&'a FieldChild>> {
		self.data
			.as_ref()
			.take_struct()
			.unwrap()
			.into_iter()
			.map(|field| SpannedValue::new(field.deref(), field.span()))
	}
}
