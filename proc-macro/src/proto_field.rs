use crate::*;

#[derive(Clone)]
pub enum ProtoField {
  Map(ProtoMap),
  Oneof(OneofInfo),
  Repeated(ProtoType),
  Optional(ProtoType),
  Single(ProtoType),
}

impl ProtoField {
  pub fn from_meta(
    ident_str: &str,
    meta: &ParseNestedMeta,
    type_info: &TypeInfo,
  ) -> syn::Result<Self> {
    let output = match ident_str {
      "repeated" => {
        let inner_type_info = if let RustType::Vec(inner) = type_info.type_.as_ref() {
          Some(inner.as_ref())
        } else {
          None
        };

        let inner = meta.parse_inner_value(|meta| {
          let inner_ident = meta.path.require_ident()?.to_string();

          ProtoType::from_nested_meta(&inner_ident, &meta, inner_type_info)
        })?;

        Self::Repeated(inner)
      }
      "optional" => {
        let inner_type_info = if let RustType::Option(inner) = type_info.type_.as_ref() {
          Some(inner.as_ref())
        } else {
          None
        };

        let inner = meta.parse_inner_value(|meta| {
          let inner_ident = meta.path.require_ident()?.to_string();

          ProtoType::from_nested_meta(&inner_ident, &meta, inner_type_info)
        })?;

        Self::Optional(inner)
      }
      "map" => {
        let map = parse_map_with_context(meta, &type_info.type_)?;

        Self::Map(map)
      }
      "oneof" => Self::Oneof(OneofInfo::parse(meta, type_info)?),
      _ => {
        let inferred_type = ProtoType::from_nested_meta(ident_str, meta, Some(type_info))?;

        Self::Single(inferred_type)
      }
    };

    Ok(output)
  }

  // This one has to stay with `ProtoField` because it's used by the extension
  // macro which does not create FieldData
  pub fn proto_field_target_type(&self, span: Span) -> TokenStream2 {
    let target_type = match self {
      Self::Map(map) => {
        let keys = map.keys.into_type().field_proto_type_tokens(span);
        let values = map.values.field_proto_type_tokens(span);

        if map.is_btree_map {
          quote_spanned! {span=> BTreeMap<#keys, #values> }
        } else {
          quote_spanned! {span=> HashMap<#keys, #values> }
        }
      }
      Self::Oneof { .. } => quote! {
         compile_error!("Proto type tokens should not be called for oneofs, if you see this please report it as a bug")
      },
      Self::Repeated(proto_type) => {
        let inner = proto_type.field_proto_type_tokens(span);

        quote_spanned! {span=> Vec<#inner> }
      }
      Self::Optional(proto_type) => {
        let inner = proto_type.field_proto_type_tokens(span);

        quote_spanned! {span=> Option<#inner> }
      }
      Self::Single(proto_type) => proto_type.field_proto_type_tokens(span),
    };

    quote_spanned! {span=> <#target_type as ::prelude::AsProtoField>::as_proto_field() }
  }

  // This one has to stay with `ProtoField` because it's used before
  // FieldData is fully created
  pub fn default_validator_expr(&self, span: Span) -> Option<ValidatorTokens> {
    let expr = match self {
      Self::Map(map) => {
        // Only offers a default if values are messages
        if let ProtoType::Message(MessageInfo { path, .. }) = &map.values {
          let keys_type = map.keys.into_type().validator_target_type(span);

          Some(quote_spanned! {span=>
            MapValidator::<#keys_type, #path>::default()
          })
        } else {
          None
        }
      }
      Self::Repeated(inner) => {
        // Only offers a default if the items are messages
        if let ProtoType::Message(MessageInfo { path, .. }) = inner {
          Some(quote_spanned! {span=>
            RepeatedValidator::<#path>::default()
          })
        } else {
          None
        }
      }
      Self::Optional(inner) | Self::Single(inner) => {
        if let ProtoType::Message(MessageInfo { path, .. }) = inner {
          Some(quote_spanned! {span=>
            MessageValidator::<#path>::default()
          })
        } else {
          None
        }
      }
      _ => None,
    };

    expr.map(|expr| ValidatorTokens {
      expr,
      is_fallback: true,
      span,
    })
  }

  // This one has to stay with `ProtoField` because it's used before
  // FieldData is fully created
  pub fn validator_target_type(&self, span: Span) -> TokenStream2 {
    match self {
      Self::Map(map) => {
        let keys = map.keys.into_type().validator_target_type(span);
        let values = map.values.validator_target_type(span);

        if map.is_btree_map {
          quote_spanned! {span=> BTreeMap<#keys, #values> }
        } else {
          quote_spanned! {span=> ::std::collections::HashMap<#keys, #values> }
        }
      }
      Self::Oneof { .. } => quote! {
        compile_error!("validator target type should not be triggered for oneofs, please report the bug if you see this")
      },
      Self::Repeated(proto_type) => {
        let inner = proto_type.validator_target_type(span);

        quote_spanned! {span=> Vec<#inner> }
      }
      Self::Optional(proto_type) | Self::Single(proto_type) => {
        proto_type.validator_target_type(span)
      }
    }
  }

  pub fn default_into_proto(&self, base_ident: &TokenStream2) -> TokenStream2 {
    let span = base_ident.span();

    match self {
      Self::Oneof(OneofInfo { default, .. }) => {
        if *default {
          quote_spanned! {span=> Some(#base_ident.into()) }
        } else {
          quote_spanned! {span=> #base_ident.map(|v| v.into()) }
        }
      }
      Self::Map(ProtoMap { .. }) => {
        quote_spanned! {span=> #base_ident.into_iter().map(|(k, v)| (k.into(), v.into())).collect() }
      }
      Self::Repeated(_) => {
        quote_spanned! {span=> #base_ident.into_iter().map(Into::into).collect() }
      }
      Self::Optional(inner) => {
        let conversion = if let ProtoType::Message(MessageInfo { boxed: true, .. }) = inner {
          quote_spanned! {span=> Box::new((*v)).into() }
        } else {
          quote_spanned! {span=> v.into() }
        };

        quote_spanned! {span=> #base_ident.map(|v| #conversion) }
      }
      // If a message is with `default`, then it would be processed here
      Self::Single(inner) => {
        if let ProtoType::Message(MessageInfo { boxed, default, .. }) = inner {
          let conversion = if *boxed {
            quote_spanned! {span=> Box::new((*#base_ident).into()) }
          } else {
            quote_spanned! {span=> #base_ident.into() }
          };

          if *default {
            quote_spanned! {span=> Some(#conversion) }
          } else {
            conversion
          }
        } else {
          quote_spanned! {span=> #base_ident.into() }
        }
      }
    }
  }

  pub fn default_from_proto(&self, base_ident: &TokenStream2) -> TokenStream2 {
    let span = base_ident.span();

    match self {
      Self::Oneof(OneofInfo { default, .. }) => {
        if *default {
          quote_spanned! {span=> #base_ident.unwrap_or_default().into() }
        } else {
          quote_spanned! {span=> #base_ident.map(|v| v.into()) }
        }
      }
      Self::Map(ProtoMap { values, .. }) => {
        let base_ident2 = quote_spanned! {span=> v };
        let values_converter = values.default_from_proto(&base_ident2);

        quote_spanned! {span=> #base_ident.into_iter().map(|(k, v)| (k.into(), #values_converter)).collect() }
      }
      Self::Repeated(proto_type) => {
        let base_ident2 = quote_spanned! {span=> v };
        let inner = proto_type.default_from_proto(&base_ident2);

        quote_spanned! {span=> #base_ident.into_iter().map(|v| #inner).collect() }
      }
      Self::Optional(proto_type) => {
        let base_ident2 = quote_spanned! {span=> v };
        let inner = proto_type.default_from_proto(&base_ident2);

        quote_spanned! {span=> #base_ident.map(|v| #inner) }
      }
      // If a message is with `default`, then it would be processed here
      Self::Single(proto_type) => proto_type.default_from_proto(base_ident),
    }
  }

  pub const fn inner(&self) -> Option<&ProtoType> {
    match self {
      Self::Map(_) | Self::Oneof(_) => None,
      Self::Repeated(inner) | Self::Optional(inner) | Self::Single(inner) => Some(inner),
    }
  }

  pub fn is_enum(&self) -> bool {
    self.inner().is_some_and(|inner| inner.is_enum())
  }

  pub fn is_message(&self) -> bool {
    self
      .inner()
      .is_some_and(|inner| inner.is_message())
  }

  pub const fn is_oneof(&self) -> bool {
    matches!(self, Self::Oneof(..))
  }
}
