use std::borrow::Cow;

use darling::{
	ast,
	util::{Flag, Override},
	FromDeriveInput, FromField, FromMeta,
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use serde_derive_internals::{ast::Container, Ctxt, Derive};
use syn::{parse_quote, spanned::Spanned, Ident, Path, Type};

mod infer_type;

pub trait FieldBase {
	fn ident(&self) -> &Ident;
	fn ty(&self) -> &Type;
}

/// Parsed representation of a field that should be expressed as an XML attribute.
pub struct FieldAttribute {
	pub ident: Ident,
	pub ty: Type,
	pub key: String,
	pub optional: bool,
	pub is_string: bool,
	span: Span,
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

impl FieldBase for FieldAttribute {
	fn ident(&self) -> &Ident {
		&self.ident
	}

	fn ty(&self) -> &Type {
		&self.ty
	}
}

impl Spanned for FieldAttribute {
	fn span(&self) -> Span {
		self.span
	}
}

pub struct FieldChild {
	pub ident: Ident,
	pub ty: Type,
	span: Span,
}

impl FieldBase for FieldChild {
	fn ident(&self) -> &Ident {
		&self.ident
	}

	fn ty(&self) -> &Type {
		&self.ty
	}
}

impl Spanned for FieldChild {
	fn span(&self) -> Span {
		self.span
	}
}

impl FromField for FieldChild {
	fn from_field(field: &syn::Field) -> darling::Result<Self> {
		if let Some(ident) = field.ident.clone() {
			Ok(Self {
				ident,
				ty: field.ty.clone(),
				span: field.span(),
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

impl FieldBase for Field {
	fn ident(&self) -> &Ident {
		match self {
			Field::Attribute(v) => v.ident(),
			Field::Child(v) => v.ident(),
		}
	}

	fn ty(&self) -> &Type {
		match self {
			Field::Attribute(v) => v.ty(),
			Field::Child(v) => v.ty(),
		}
	}
}

impl FromField for Field {
	fn from_field(field: &syn::Field) -> darling::Result<Self> {
		#[derive(Default, FromMeta)]
		#[darling(default)]
		struct RawFieldAttr {
			pub key: Option<String>,
			pub optional: Option<bool>,
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
				is_string: infer_type::as_ref_str(&ty),
				optional: attr.optional.unwrap_or_else(|| infer_type::option(&ty)),
				ident,
				ty,
				span: field.span(),
			}))
		} else if attr_string.is_some() {
			Ok(Field::Attribute(FieldAttribute {
				key: ident.to_string(),
				is_string: true,
				optional: false,
				ident,
				ty,
				span: field.span(),
			}))
		} else {
			Ok(Field::Child(FieldChild {
				ident,
				ty,
				span: field.span(),
			}))
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
	pub data: ast::Data<(), Field>,
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
	pub fn fields<'a>(&'a self) -> impl Iterator<Item = &'a Field> {
		self.data.as_ref().take_struct().unwrap().into_iter()
	}

	/// Fields of the input struct that are represented as attributes.
	pub fn attrs<'a>(&'a self) -> impl Iterator<Item = &'a FieldAttribute> {
		self.fields().filter_map(|field| {
			if let Field::Attribute(attr) = field {
				Some(attr)
			} else {
				None
			}
		})
	}

	/// Fields of the input struct that are represented as child elements.
	pub fn elements<'a>(&'a self) -> impl Iterator<Item = &'a FieldChild> {
		self.fields().filter_map(|field| {
			if let Field::Child(child) = field {
				Some(child)
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
	pub data: ast::Data<(), FieldChild>,
	#[darling(rename = "crate", default = "default_crate_path")]
	pub xml_data_crate: Path,
}

impl InnerInput {
	pub fn elements<'a>(&'a self) -> impl Iterator<Item = &'a FieldChild> {
		self.data.as_ref().take_struct().unwrap().into_iter()
	}
}

pub struct SerdeElementInput<'a> {
	pub xml: ElementInput,
	pub serde: Container<'a>,
}

impl<'a> SerdeElementInput<'a> {
	pub fn new(
		input: &'a syn::DeriveInput,
		serde_ctx: &'a Ctxt,
	) -> Result<Self, Option<darling::Error>> {
		let xml = ElementInput::from_derive_input(input)?;
		let serde = Container::from_ast(serde_ctx, input, Derive::Serialize).ok_or(None)?;

		Ok(Self { xml, serde })
	}

	pub fn fields(&'a self) -> impl Iterator<Item = SField<&'a Field>> {
		self.xml
			.fields()
			.zip(self.serde.data.all_fields())
			.map(move |(x, s)| SField {
				parent: self,
				xml: x,
				serde: s,
			})
	}

	pub fn attrs(&'a self) -> impl Iterator<Item = SField<&'a FieldAttribute>> {
		self.fields().filter_map(|f| f.as_field_attr())
	}

	pub fn children(&'a self) -> impl Iterator<Item = SField<&'a FieldChild>> {
		self.fields().filter_map(|f| f.as_field_child())
	}
}

pub struct SField<'a, T> {
	parent: &'a SerdeElementInput<'a>,
	pub xml: T,
	pub serde: &'a serde_derive_internals::ast::Field<'a>,
}

impl<'a, T> SField<'a, T> {
	pub fn xml_data_crate(&self) -> &Path {
		&self.parent.xml.xml_data_crate
	}
}

impl<'a> SField<'a, &'a Field> {
	pub fn as_field_attr(&self) -> Option<SField<'a, &'a FieldAttribute>> {
		if let Field::Attribute(attr) = self.xml {
			Some(SField {
				parent: self.parent,
				xml: attr,
				serde: self.serde,
			})
		} else {
			None
		}
	}

	pub fn as_field_child(&self) -> Option<SField<'a, &'a FieldChild>> {
		if let Field::Child(child) = self.xml {
			Some(SField {
				parent: self.parent,
				xml: child,
				serde: self.serde,
			})
		} else {
			None
		}
	}
}

impl<'a, T: Spanned> Spanned for SField<'a, &'a T> {
	fn span(&self) -> Span {
		self.xml.span()
	}
}

impl<'a, T: FieldBase> FieldBase for SField<'a, &'a T> {
	fn ident(&self) -> &Ident {
		self.xml.ident()
	}

	fn ty(&self) -> &Type {
		self.xml.ty()
	}
}
