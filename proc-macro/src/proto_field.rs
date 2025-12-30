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

        let inner = meta
          .parse_inner_value(|meta| {
            let inner_ident = meta.path.require_ident()?.to_string();

            ProtoType::from_nested_meta(&inner_ident, meta, fallback.as_ref())
          })?
          .ok_or_else(|| meta.error("Expected a path or list"))?;

        Self::Repeated(inner)
      }
      "optional" => {
        let fallback = if let RustType::Option(inner) = type_info.type_.as_ref() {
          inner.as_path()
        } else {
          None
        };

        let inner = meta
          .parse_inner_value(|meta| {
            let inner_ident = meta.path.require_ident()?.to_string();

            ProtoType::from_nested_meta(&inner_ident, meta, fallback.as_ref())
          })?
          .ok_or_else(|| meta.error("Expected a path or list"))?;

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

  pub fn default_validator_expr(&self) -> Option<TokenStream2> {
    match self {
      Self::Map(map) => {
        if let ProtoType::Message { path, .. } = &map.values {
          let keys_type = map.keys.validator_target_type();

          Some(quote! {
            MapValidator::<#keys_type, #path>::default()
          })
        } else {
          None
        }
      }
      Self::Repeated(inner) => {
        if let ProtoType::Message { path, .. } = inner {
          Some(quote! {
            RepeatedValidator::<#path>::default()
          })
        } else {
          None
        }
      }
      Self::Optional(inner) | Self::Single(inner) => {
        if let ProtoType::Message { path, .. } = inner {
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
      ProtoField::Map(_) => quote! { ::proto_types::field_descriptor_proto::Type::Message },
      ProtoField::Repeated(inner) => inner.descriptor_type_tokens(),
      ProtoField::Optional(inner) => inner.descriptor_type_tokens(),
      ProtoField::Single(inner) => inner.descriptor_type_tokens(),
      ProtoField::Oneof { .. } => {
        quote! { compile_error!("Validator tokens should not be triggered for a oneof field") }
      }
    }
  }

  pub fn as_prost_attr(&self, tag: i32) -> Attribute {
    let inner = match self {
      ProtoField::Oneof(OneofInfo { path, tags, .. }) => {
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

      ProtoField::Repeated(proto_type) => {
        let p_type = proto_type.as_prost_attr_type();

        quote! { #p_type, repeated }
      }
      ProtoField::Optional(proto_type) => {
        let p_type = proto_type.as_prost_attr_type();

        quote! { #p_type, optional }
      }
      ProtoField::Single(proto_type) => proto_type.as_prost_attr_type(),
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
      ProtoField::Map(ProtoMap { .. }) => {
        quote! { #base_ident.into_iter().map(|(k, v)| (k.into(), v.into())).collect() }
      }
      ProtoField::Repeated(_) => {
        quote! { #base_ident.into_iter().map(Into::into).collect() }
      }
      ProtoField::Optional(inner) => {
        let conversion = if inner.is_message() {
          let base_ident2 = quote! { v };
          inner.default_into_proto(&base_ident2)
        } else {
          quote! { v.into() }
        };

        quote! { #base_ident.map(|v| #conversion) }
      }
      ProtoField::Single(proto_type) => proto_type.default_into_proto(base_ident),
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
      ProtoField::Map(ProtoMap { values, .. }) => {
        let base_ident2 = quote! { v };
        let values_converter = values.default_from_proto(&base_ident2);

        quote! { #base_ident.into_iter().map(|(k, v)| (k.into(), #values_converter)).collect() }
      }
      ProtoField::Repeated(proto_type) => {
        let base_ident2 = quote! { v };
        let inner = proto_type.default_from_proto(&base_ident2);

        quote! { #base_ident.into_iter().map(|v| #inner).collect() }
      }
      ProtoField::Optional(proto_type) => {
        let base_ident2 = quote! { v };
        let inner = proto_type.default_from_proto(&base_ident2);

        quote! { #base_ident.map(|v| #inner) }
      }
      ProtoField::Single(proto_type) => proto_type.default_from_proto(base_ident),
    }
  }

  pub fn validator_target_type(&self) -> TokenStream2 {
    match self {
      Self::Map(map) => {
        let keys = map.keys.validator_target_type();
        let values = map.values.validator_target_type();

        quote! { ::prelude::ProtoMap<#keys, #values> }
      }
      ProtoField::Oneof { .. } => quote! {},
      ProtoField::Repeated(proto_type) => {
        let inner = proto_type.validator_target_type();

        quote! { Vec<#inner> }
      }
      ProtoField::Optional(proto_type) => proto_type.validator_target_type(),
      ProtoField::Single(proto_type) => proto_type.validator_target_type(),
    }
  }

  pub fn validator_name(&self) -> TokenStream2 {
    match self {
      Self::Map(map) => {
        let keys = map.keys.validator_target_type();
        let values = map.values.validator_target_type();

        quote! { MapValidator<#keys, #values> }
      }
      ProtoField::Oneof { .. } => quote! {},
      ProtoField::Repeated(proto_type) => {
        let inner = proto_type.validator_target_type();

        quote! { RepeatedValidator<#inner> }
      }
      ProtoField::Optional(proto_type) => proto_type.validator_name(),
      ProtoField::Single(proto_type) => proto_type.validator_name(),
    }
  }

  pub fn field_proto_type_tokens(&self) -> TokenStream2 {
    let target_type = match self {
      ProtoField::Map(proto_map) => {
        let keys = proto_map.keys.field_proto_type_tokens();
        let values = proto_map.values.field_proto_type_tokens();

        quote! { ::prelude::ProtoMap<#keys, #values> }
      }
      ProtoField::Oneof { .. } => quote! {},
      ProtoField::Repeated(proto_type) => {
        let inner = proto_type.field_proto_type_tokens();

        quote! { Vec<#inner> }
      }
      ProtoField::Optional(proto_type) => {
        let inner = proto_type.field_proto_type_tokens();

        quote! { Option<#inner> }
      }
      ProtoField::Single(proto_type) => proto_type.field_proto_type_tokens(),
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
      ProtoField::Repeated(inner) => {
        let inner_type = inner.output_proto_type();

        parse_quote! { Vec<#inner_type> }
      }
      ProtoField::Optional(inner) => {
        let inner_type = inner.output_proto_type();

        parse_quote! { Option<#inner_type> }
      }
      ProtoField::Single(inner) => {
        let inner = inner.output_proto_type();

        parse_quote! { #inner }
      }
    }
  }
}
