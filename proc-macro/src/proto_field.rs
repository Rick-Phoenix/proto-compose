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
    meta: ParseNestedMeta,
    type_info: &TypeInfo,
  ) -> syn::Result<Self> {
    let output = match ident_str {
      "repeated" => {
        let fallback = if let RustType::Vec(inner) = type_info.type_.as_ref() {
          inner.as_path()
        } else {
          None
        };

        let inner = meta.parse_inner_value(|meta| {
          let inner_ident = meta.path.require_ident()?.to_string();

          ProtoType::from_nested_meta(&inner_ident, meta, fallback.as_ref())
        })?;

        Self::Repeated(inner)
      }
      "optional" => {
        let fallback = if let RustType::Option(inner) = type_info.type_.as_ref() {
          inner.as_path()
        } else {
          None
        };

        let inner = meta.parse_inner_value(|meta| {
          let inner_ident = meta.path.require_ident()?.to_string();

          ProtoType::from_nested_meta(&inner_ident, meta, fallback.as_ref())
        })?;

        Self::Optional(inner)
      }
      "map" => {
        let map = parse_map_with_context(meta, &type_info.type_)?;

        Self::Map(map)
      }
      "oneof" => Self::Oneof(OneofInfo::parse(meta, type_info)?),
      _ => {
        let inner =
          ProtoType::from_nested_meta(ident_str, meta, type_info.inner().as_path().as_ref())?;

        Self::Single(inner)
      }
    };

    Ok(output)
  }

  // Maybe I should handle the oneof default here?
  pub fn default_validator_expr(&self) -> Option<TokenStream2> {
    match self {
      Self::Map(map) => {
        if let ProtoType::Message(MessageInfo { path, .. }) = &map.values {
          let keys_type = map.keys.validator_target_type();

          Some(quote! {
            MapValidator::<#keys_type, #path>::default()
          })
        } else {
          None
        }
      }
      Self::Repeated(inner) => {
        if let ProtoType::Message(MessageInfo { path, .. }) = inner {
          Some(quote! {
            RepeatedValidator::<#path>::default()
          })
        } else {
          None
        }
      }
      Self::Optional(inner) | Self::Single(inner) => {
        if let ProtoType::Message(MessageInfo { path, .. }) = inner {
          Some(quote! {
            MessageValidator::<#path>::default()
          })
        } else {
          None
        }
      }
      _ => None,
    }
  }

  pub fn descriptor_type_tokens(&self) -> TokenStream2 {
    match self {
      Self::Map(_) => {
        quote! { ::prelude::proto_types::field_descriptor_proto::Type::Message }
      }
      Self::Repeated(inner) | Self::Optional(inner) | Self::Single(inner) => {
        inner.descriptor_type_tokens()
      }
      Self::Oneof { .. } => {
        quote! { compile_error!("Validator tokens should not be triggered for a oneof field") }
      }
    }
  }

  pub fn as_prost_attr(&self, tag: i32) -> Attribute {
    let inner = match self {
      Self::Oneof(OneofInfo { path, tags, .. }) => {
        let oneof_path_str = path.to_token_stream().to_string();
        let tags_str = tags_to_str(tags);

        // We don't need to add the tag for oneofs,
        // so we return early
        return parse_quote! { #[prost(oneof = #oneof_path_str, tags = #tags_str)] };
      }
      Self::Map(map) => {
        let map_attr = format!("{}, {}", map.keys, map.values.as_prost_map_value());

        quote! { map = #map_attr }
      }

      Self::Repeated(proto_type) => {
        let p_type = proto_type.as_prost_attr_type();

        quote! { #p_type, repeated }
      }
      Self::Optional(proto_type) => {
        let p_type = proto_type.as_prost_attr_type();

        quote! { #p_type, optional }
      }
      Self::Single(proto_type) => proto_type.as_prost_attr_type(),
    };

    let tag_as_str = tag.to_string();

    parse_quote! { #[prost(#inner, tag = #tag_as_str)] }
  }

  pub fn default_into_proto(&self, base_ident: &TokenStream2) -> TokenStream2 {
    match self {
      Self::Oneof(OneofInfo { default, .. }) => {
        if *default {
          quote! { Some(#base_ident.into()) }
        } else {
          quote! { #base_ident.map(|v| v.into()) }
        }
      }
      Self::Map(ProtoMap { .. }) => {
        quote! { #base_ident.into_iter().map(|(k, v)| (k.into(), v.into())).collect() }
      }
      Self::Repeated(_) => {
        quote! { #base_ident.into_iter().map(Into::into).collect() }
      }
      Self::Optional(inner) => {
        let conversion = if inner.is_message() {
          let base_ident2 = quote! { v };
          inner.default_into_proto(&base_ident2)
        } else {
          quote! { v.into() }
        };

        quote! { #base_ident.map(|v| #conversion) }
      }
      Self::Single(proto_type) => proto_type.default_into_proto(base_ident),
    }
  }

  pub fn default_from_proto(&self, base_ident: &TokenStream2) -> TokenStream2 {
    match self {
      Self::Oneof(OneofInfo { default, .. }) => {
        if *default {
          quote! { #base_ident.unwrap_or_default().into() }
        } else {
          quote! { #base_ident.map(|v| v.into()) }
        }
      }
      Self::Map(ProtoMap { values, .. }) => {
        let base_ident2 = quote! { v };
        let values_converter = values.default_from_proto(&base_ident2);

        quote! { #base_ident.into_iter().map(|(k, v)| (k.into(), #values_converter)).collect() }
      }
      Self::Repeated(proto_type) => {
        let base_ident2 = quote! { v };
        let inner = proto_type.default_from_proto(&base_ident2);

        quote! { #base_ident.into_iter().map(|v| #inner).collect() }
      }
      Self::Optional(proto_type) => {
        let base_ident2 = quote! { v };
        let inner = proto_type.default_from_proto(&base_ident2);

        quote! { #base_ident.map(|v| #inner) }
      }
      Self::Single(proto_type) => proto_type.default_from_proto(base_ident),
    }
  }

  pub fn validator_target_type(&self) -> TokenStream2 {
    match self {
      Self::Map(map) => {
        let keys = map.keys.validator_target_type();
        let values = map.values.validator_target_type();

        quote! { ::prelude::ProtoMap<#keys, #values> }
      }
      Self::Oneof { .. } => quote! {},
      Self::Repeated(proto_type) => {
        let inner = proto_type.validator_target_type();

        quote! { Vec<#inner> }
      }
      Self::Optional(proto_type) | Self::Single(proto_type) => proto_type.validator_target_type(),
    }
  }

  pub fn validator_name(&self) -> TokenStream2 {
    match self {
      Self::Map(map) => {
        let keys = map.keys.validator_target_type();
        let values = map.values.validator_target_type();

        quote! { MapValidator<#keys, #values> }
      }
      Self::Oneof { .. } => quote! {},
      Self::Repeated(proto_type) => {
        let inner = proto_type.validator_target_type();

        quote! { RepeatedValidator<#inner> }
      }
      Self::Optional(proto_type) | Self::Single(proto_type) => proto_type.validator_name(),
    }
  }

  pub fn field_proto_type_tokens(&self) -> TokenStream2 {
    let target_type = match self {
      Self::Map(proto_map) => {
        let keys = proto_map.keys.field_proto_type_tokens();
        let values = proto_map.values.field_proto_type_tokens();

        quote! { ::prelude::ProtoMap<#keys, #values> }
      }
      Self::Oneof { .. } => quote! {},
      Self::Repeated(proto_type) => {
        let inner = proto_type.field_proto_type_tokens();

        quote! { Vec<#inner> }
      }
      Self::Optional(proto_type) => {
        let inner = proto_type.field_proto_type_tokens();

        quote! { Option<#inner> }
      }
      Self::Single(proto_type) => proto_type.field_proto_type_tokens(),
    };

    quote! { <#target_type as ::prelude::AsProtoField>::as_proto_field() }
  }

  pub fn output_proto_type(&self) -> Type {
    match self {
      Self::Map(map) => {
        let keys = map.keys.output_proto_type();
        let values = map.values.output_proto_type();

        parse_quote! { std::collections::HashMap<#keys, #values> }
      }
      Self::Oneof(OneofInfo { path, .. }) => parse_quote! { Option<#path> },
      Self::Repeated(inner) => {
        let inner_type = inner.output_proto_type();

        parse_quote! { Vec<#inner_type> }
      }
      Self::Optional(inner) => {
        let inner_type = inner.output_proto_type();

        parse_quote! { Option<#inner_type> }
      }
      Self::Single(inner) => {
        let inner = inner.output_proto_type();

        parse_quote! { #inner }
      }
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

  pub fn is_boxed_message(&self) -> bool {
    self
      .inner()
      .is_some_and(|inner| inner.is_boxed_message())
  }

  pub const fn is_oneof(&self) -> bool {
    matches!(self, Self::Oneof(..))
  }
}
