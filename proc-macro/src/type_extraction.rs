use std::{borrow::Cow, fmt::Display};

use syn::spanned::Spanned;

use crate::*;

#[derive(Clone)]
pub struct TypeInfo {
  pub rust_type: RustType,
  pub span: Span,
  pub proto_type: ProtoType,
}

impl TypeInfo {
  pub fn into_proto(&self, base_ident: TokenStream2) -> TokenStream2 {
    if let ProtoType::Oneof {
      default: true,
      path,
      is_proxied,
      ..
    } = &self.proto_type
    {
      if *is_proxied {
        return quote! { #base_ident.into().into() };
      } else {
        return quote! { #base_ident.unwrap_or_default() };
      }
    }

    match &self.rust_type {
      RustType::Option(_) => quote! { #base_ident.map(Into::into) },
      RustType::BoxedMsg(_) => quote! { #base_ident.map(|v| Box::new((*v).into())) },
      RustType::Map(_) => quote! { #base_ident.into_iter().map(|(k, v)| (k, v.into())).collect() },
      RustType::Vec(_) => quote! { #base_ident.into_iter().map(Into::into).collect() },
      RustType::Normal(_) => quote! { #base_ident.into() },
      RustType::BoxedOneofVariant(_) => quote! { Box::new((*#base_ident).into()) },
    }
  }

  pub fn from_proto(&self, base_ident: TokenStream2) -> TokenStream2 {
    let conversion_call = match &self.rust_type {
      RustType::Normal(_) | RustType::BoxedOneofVariant(_) => {
        self.proto_type.default_from_proto(&base_ident)
      }
      _ => {
        let inner_base_ident = quote! { v };

        self.proto_type.default_from_proto(&inner_base_ident)
      }
    };

    match &self.rust_type {
      RustType::Option(_) => quote! { #base_ident.map(|v| #conversion_call) },
      RustType::BoxedMsg(_) => quote! { #base_ident.map(|v| Box::new((*v).into())) },
      RustType::Map(_) => {
        let value_conversion = if let ProtoType::Map(map) = &self.proto_type && map.has_enum_values() {
            quote! { try_into().unwrap_or_default() }
          } else {
            quote! { into() }
          };

        quote! { #base_ident.into_iter().map(|(k, v)| (k, v.#value_conversion)).collect() }
      }
      RustType::Vec(_) => quote! { #base_ident.into_iter().map(|v| #conversion_call).collect() },
      _ => conversion_call,
    }
  }

  pub fn validator_tokens(
    &self,
    validator: &ValidatorExpr,
    proto_type: &ProtoType,
  ) -> TokenStream2 {
    let mut target_type = proto_type.validator_target_type();

    if matches!(self.rust_type, RustType::Vec(_)) {
      target_type = quote! { Vec<#target_type> };
    }

    match validator {
      ValidatorExpr::Call(call) => {
        quote! { Some(<ValidatorMap as ProtoValidator<#target_type>>::from_builder(#call)) }
      }

      ValidatorExpr::Closure(closure) => {
        quote! { Some(<ValidatorMap as ProtoValidator<#target_type>>::build_rules(#closure)) }
      }
    }
  }

  pub fn as_proto_type_trait_expr(&self, proto_type: &ProtoType) -> TokenStream2 {
    let base_target_type = proto_type.as_proto_type_trait_target();

    let target_type = match &self.rust_type {
      RustType::Option(_) => quote! { Option<#base_target_type> },
      RustType::BoxedMsg(_) => quote! { Option<#base_target_type> },
      RustType::Map(_) => base_target_type,
      RustType::Vec(_) => quote! { Vec<#base_target_type> },
      RustType::Normal(_) => base_target_type,
      RustType::BoxedOneofVariant(_) => base_target_type,
    };

    quote! { <#target_type as AsProtoType>::proto_type() }
  }

  pub fn is_option(&self) -> bool {
    matches!(self.rust_type, RustType::Option(_))
  }

  pub fn as_inner_option_path(&self) -> Option<&Path> {
    if let RustType::Option(path) = &self.rust_type {
      Some(path)
    } else {
      None
    }
  }

  pub fn from_type(
    ty: &Type,
    field_kind: ProtoFieldKind,
    item_ident: &Ident,
  ) -> Result<Self, Error> {
    let path = extract_type_path(ty)?;
    let rust_type = RustType::from_path(path, item_ident);

    let proto_type = extract_proto_type(&rust_type, field_kind, ty)?;

    let span = ty.span();

    Ok(Self {
      rust_type,
      span,
      proto_type,
    })
  }

  pub fn error(&self, error: impl Display) -> Error {
    syn::Error::new(self.span, error)
  }
}
