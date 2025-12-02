use crate::*;

#[derive(Clone)]
pub enum ProtoField {
  Map(ProtoMap),
  Oneof {
    path: Path,
    tags: Vec<i32>,
    default: bool,
  },
  Repeated(ProtoType),
  Optional(ProtoType),
  Single(ProtoType),
}

impl ProtoField {
  pub fn as_prost_attr_type(&self) -> TokenStream2 {
    match self {
      Self::Map(map) => map.as_prost_attr_type(),
      ProtoField::Oneof { path, tags, .. } => {
        let oneof_path_str = path.to_token_stream().to_string();
        let tags_str = tags_to_str(tags);

        quote! { oneof = #oneof_path_str, tags = #tags_str }
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
    }
  }

  pub fn default_into_proto(&self, base_ident: &TokenStream2) -> TokenStream2 {
    match self {
      Self::Oneof { default, .. } => {
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
      Self::Oneof { default, .. } => {
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
      Self::Map(map) => map.validator_target_type(),
      ProtoField::Oneof { .. } => quote! {},
      ProtoField::Repeated(proto_type) => {
        let inner = proto_type.validator_target_type();

        quote! { Vec<#inner> }
      }
      ProtoField::Optional(proto_type) => proto_type.validator_target_type(),
      ProtoField::Single(proto_type) => proto_type.validator_target_type(),
    }
  }

  pub fn as_proto_type_trait_expr(&self) -> TokenStream2 {
    let target_type = self.output_proto_type(false);

    quote! { <#target_type as AsProtoType>::proto_type() }
  }

  pub fn output_proto_type(&self, is_oneof_variant: bool) -> TokenStream2 {
    match self {
      Self::Map(map) => map.output_proto_type(),
      Self::Oneof { path, .. } => quote! { Option<#path> },
      ProtoField::Repeated(inner) => {
        let inner_type = inner.output_proto_type();

        quote! { Vec<#inner_type> }
      }
      ProtoField::Optional(inner) => {
        let inner_type = inner.output_proto_type();

        quote! { Option<#inner_type> }
      }
      ProtoField::Single(inner) => {
        if let ProtoType::Message { path, is_boxed, .. } = inner {
          if *is_boxed {
            if is_oneof_variant {
              quote! { Box<#path> }
            } else {
              quote! { Option<Box<#path>> }
            }
          } else {
            quote! { Option<#path> }
          }
        } else {
          inner.output_proto_type()
        }
      }
    }
  }

  /// Returns `true` if the proto field is [`Oneof`].
  ///
  /// [`Oneof`]: ProtoField::Oneof
  #[must_use]
  pub fn is_oneof(&self) -> bool {
    matches!(self, Self::Oneof { .. })
  }
}
