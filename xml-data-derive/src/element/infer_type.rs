use syn::{GenericArgument, PathArguments, PathSegment, Type};

/// Infer whether a type is likely one that is known to implement `AsRef<str>`.
pub fn as_ref_str(ty: &Type) -> bool {
	if let Some(last) = selfless_last(ty) {
		if last.ident == "String" {
			return true;
		}

		if last.ident == "Cow" {
			if let PathArguments::AngleBracketed(args) = &last.arguments {
				if args.args.len() != 2 {
					return false;
				}

				return match args.args.iter().nth(1) {
					Some(GenericArgument::Type(gat)) => as_ref_str(gat),
					Some(_) | None => false,
				};
			}
		}
	}

	false
}

/// Infer whether a type appears to be `Option<T>`.
pub fn option(ty: &Type) -> bool {
	if let Some(last) = selfless_last(ty) {
		if last.ident != "Option" {
			return false;
		}

		if let PathArguments::AngleBracketed(args) = &last.arguments {
			return args.args.len() == 1;
		}
	}

	false
}

fn selfless_last(ty: &Type) -> Option<&PathSegment> {
	if let Type::Path(ty) = ty {
		if ty.qself.is_none() {
			return ty.path.segments.last();
		}
	}

	None
}
