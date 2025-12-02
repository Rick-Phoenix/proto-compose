use crate::*;

pub struct IntoProto<'a> {
  pub custom_expression: &'a Option<PathOrClosure>,
  pub kind: FieldConversionKind<'a>,
  pub type_info: &'a TypeInfo,
}

pub struct FromProto<'a> {
  pub custom_expression: &'a Option<PathOrClosure>,
  pub kind: FieldConversionKind<'a>,
  pub type_info: Option<&'a TypeInfo>,
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

pub fn field_into_proto_expression(info: IntoProto) -> Result<TokenStream2, Error> {
  let IntoProto {
    custom_expression,
    type_info,
    kind,
  } = info;

  let mut base_ident = match kind {
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
  } else if let ProtoField::Oneof { default: true, .. } = &type_info.proto_field {
    quote! { Some(#base_ident.into()) }
  } else {
    type_info.into_proto(base_ident)
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

pub fn field_from_proto_expression(info: FromProto) -> Result<TokenStream2, Error> {
  let FromProto {
    custom_expression,
    type_info,
    kind,
  } = info;

  let base_ident = match kind {
    FieldConversionKind::EnumVariant { .. } => {
      quote! { v }
    }
    FieldConversionKind::StructField { ident } => {
      quote! { value.#ident }
    }
  };

  let conversion_expr = if let Some(type_info) = type_info {
    if let Some(expr) = custom_expression {
      match expr {
        PathOrClosure::Path(path) => quote! { #path(#base_ident) },
        PathOrClosure::Closure(closure) => {
          quote! {
            prelude::apply(#base_ident, #closure)
          }
        }
      }
    } else {
      let is_oneof = matches!(kind, FieldConversionKind::EnumVariant { .. });

      type_info.from_proto(base_ident, is_oneof)
    }
  } else {
    // Field is ignored
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

pub struct ItemConversion<'a> {
  pub source_ident: &'a Ident,
  pub target_ident: &'a Ident,
  pub kind: ItemConversionKind,
  pub custom_expression: &'a Option<PathOrClosure>,
  pub conversion_tokens: TokenStream2,
}

pub enum ItemConversionKind {
  Enum,
  Struct,
}

fn create_from_impl(info: &ItemConversion) -> TokenStream2 {
  let ItemConversion {
    source_ident,
    target_ident,
    kind,
    custom_expression,
    conversion_tokens,
  } = info;

  let conversion_body = if let Some(expr) = custom_expression {
    match expr {
      PathOrClosure::Path(path) => quote! { #path(value) },
      PathOrClosure::Closure(closure) => quote! {
        prelude::apply(value, #closure)
      },
    }
  } else {
    match kind {
      ItemConversionKind::Enum => quote! {
        match value {
          #conversion_tokens
        }
      },
      ItemConversionKind::Struct => quote! {
        Self {
          #conversion_tokens
        }
      },
    }
  };

  quote! {
    impl From<#source_ident> for #target_ident {
      fn from(value: #source_ident) -> Self {
        #conversion_body
      }
    }
  }
}

pub fn into_proto_impl(info: ItemConversion) -> TokenStream2 {
  let into_proto_impl = create_from_impl(&info);

  let ItemConversion {
    source_ident,
    target_ident,
    ..
  } = info;

  quote! {
    #into_proto_impl

    impl #source_ident {
      pub fn into_proto(self) -> #target_ident {
        self.into()
      }
    }
  }
}

pub fn from_proto_impl(info: ItemConversion) -> TokenStream2 {
  let ItemConversion {
    source_ident,
    target_ident,
    kind,
    custom_expression,
    conversion_tokens,
  } = info;

  // We use the original source and target for the helper
  let from_proto_helper = quote! {
    impl #source_ident {
      pub fn from_proto(value: #target_ident) -> Self {
        value.into()
      }
    }
  };

  // And we switch them to create the From impl from _Proto to the original item
  let switched = ItemConversion {
    source_ident: target_ident,
    target_ident: source_ident,
    kind,
    custom_expression,
    conversion_tokens,
  };

  let from_proto_impl = create_from_impl(&switched);

  quote! {
    #from_proto_impl

    #from_proto_helper
  }
}
