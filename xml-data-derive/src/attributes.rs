use syn::{
	Attribute,
	Lit,
	Meta,
	NestedMeta,
};

pub fn all_attributes(attrs: &[Attribute]) -> impl Iterator<Item = NestedMeta> + '_ {
	attrs
		.iter()
		.filter_map(|attr| {
			if attr.path.is_ident("xml_data") {
				match attr.parse_meta() {
					Ok(Meta::List(meta)) => Some(meta.nested.into_iter()),
					Ok(_) => {
						panic!("expected #[xml_data(...)]");
					},
					Err(err) => {
						panic!("#[xml_data]: {}", err);
					},
				}
			} else {
				None
			}
		})
		.flatten()
}

pub fn single_nested(meta: &Meta) -> Option<&NestedMeta> {
	match meta {
		Meta::Path(_) => None,
		Meta::List(l) => {
			if l.nested.len() > 1 {
				panic!("only single argument allowed for argument");
			}
			l.nested.first()
		},
		Meta::NameValue(_) => panic!("Expect single nested argument"),
	}
}

pub fn single_lit(meta: &Meta) -> Option<&syn::Lit> {
	match meta {
		Meta::Path(_) => None,
		Meta::List(l) => {
			if l.nested.len() > 1 {
				panic!("only single argument allowed for argument");
			}
			match l.nested.first() {
				Some(NestedMeta::Lit(l)) => Some(l),
				Some(NestedMeta::Meta(m)) => single_lit(m),
				None => None,
			}
		},
		Meta::NameValue(nv) => Some(&nv.lit),
	}
}

pub fn string_lit(meta: &Meta) -> Option<String> {
	single_lit(meta).map(|l| {
		if let Lit::Str(s) = l {
			s.value()
		} else {
			panic!("invalid literal; expected string");
		}
	})
}
