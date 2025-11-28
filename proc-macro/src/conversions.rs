use crate::*;

pub struct FieldConversion<'a> {
  pub custom_expression: &'a Option<PathOrClosure>,
  pub kind: FieldConversionKind<'a>,
  pub type_info: &'a TypeInfo,
  pub is_ignored: bool,
}

pub enum FieldConversionKind<'a> {
  StructField {
    ident: &'a Ident,
  },
  EnumVariant {
    variant_ident: &'a Ident,
    source_enum_ident: &'a Ident,
    target_enum_ident: &'a Ident,
  },
}

impl<'a> FieldConversionKind<'a> {
  #[must_use]
  pub fn is_struct_field(&self) -> bool {
    matches!(self, Self::StructField { .. })
  }

  #[must_use]
  pub fn is_enum_variant(&self) -> bool {
    matches!(self, Self::EnumVariant { .. })
  }
}

pub fn field_into_proto_expression(info: FieldConversion) -> Result<TokenStream2, Error> {
  let FieldConversion {
    custom_expression,
    type_info,
    kind,
    ..
  } = &info;

  let base_ident = match kind {
    FieldConversionKind::EnumVariant { .. } => {
      quote! { v }
    }
    FieldConversionKind::StructField { ident } => {
      quote! { value.#ident }
    }
  };

  let conversion_expr = if let Some(expr) = custom_expression {
    match expr {
      PathOrClosure::Path(path) => quote! { #path(#base_ident) },
      PathOrClosure::Closure(closure) => {
        quote! {
          prelude::apply(#base_ident, #closure)
        }
      }
    }
  } else if let ProtoType::Oneof { default: true, .. } = &type_info.proto_type {
    quote! { Some(#base_ident.into()) }
  } else {
    let call = type_info.into_proto();

    quote! { #base_ident.#call }
  };

  let conversion = match kind {
    FieldConversionKind::StructField { ident } => quote! {
      #ident: #conversion_expr,
    },
    FieldConversionKind::EnumVariant {
      variant_ident,
      source_enum_ident,
      target_enum_ident,
    } => quote! {
      #source_enum_ident::#variant_ident(v) => #target_enum_ident::#variant_ident(#conversion_expr),
    },
  };

  Ok(conversion)
}

pub fn field_from_proto_expression(info: FieldConversion) -> Result<TokenStream2, Error> {
  let FieldConversion {
    custom_expression,
    type_info,
    kind,
    is_ignored,
  } = &info;

  let base_ident = match kind {
    FieldConversionKind::EnumVariant { .. } => {
      quote! { v }
    }
    FieldConversionKind::StructField { ident } => {
      quote! { value.#ident }
    }
  };

  let conversion_expr = if *is_ignored {
    if let Some(expr) = custom_expression {
      match expr {
        PathOrClosure::Path(path) => quote! { #path() },
        PathOrClosure::Closure(closure) => {
          return Err(spanned_error!(
            closure,
            "Cannot use a closure for ignored fields"
          ))
        }
      }
    } else {
      quote! { Default::default() }
    }
  } else if let Some(expr) = custom_expression {
    match expr {
      PathOrClosure::Path(path) => quote! { #path(#base_ident) },
      PathOrClosure::Closure(closure) => {
        quote! {
          prelude::apply(#base_ident, #closure)
        }
      }
    }
  } else {
    let call = type_info.from_proto();

    quote! { #base_ident.#call }
  };

  let conversion = match kind {
    FieldConversionKind::StructField { ident } => quote! {
      #ident: #conversion_expr,
    },
    FieldConversionKind::EnumVariant {
      variant_ident,
      source_enum_ident,
      target_enum_ident,
    } => quote! {
      #target_enum_ident::#variant_ident(v) => #source_enum_ident::#variant_ident(#conversion_expr),
    },
  };

  Ok(conversion)
}
