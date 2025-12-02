use std::{borrow::Cow, fmt::Display};

use syn::spanned::Spanned;

use crate::*;

#[derive(Clone)]
pub struct TypeInfo {
  pub rust_type: RustType,
  pub span: Span,
  pub proto_field: ProtoField,
}

impl TypeInfo {
  pub fn as_prost_attr(&self, tag: i32) -> TokenStream2 {
    let type_attr = self.proto_field.as_prost_attr_type();

    if self.proto_field.is_oneof() {
      quote! { #[prost(#type_attr)] }
    } else {
      let tag_as_str = tag.to_string();

      quote! { #[prost(#type_attr, tag = #tag_as_str)] }
    }
  }

  pub fn into_proto(&self, base_ident: TokenStream2) -> TokenStream2 {
    if let ProtoField::Oneof {
      default: true,
      path,
      is_proxied,
      ..
    } = &self.proto_field
    {
      if *is_proxied {
        return quote! { #base_ident.into().into() };
      } else {
        return quote! { #base_ident.unwrap_or_default() };
      }
    }

    match &self.rust_type {
      RustType::Option(_) => quote! { #base_ident.map(Into::into) },
      RustType::OptionBoxed(_) => quote! { #base_ident.map(|v| Box::new((*v).into())) },
      RustType::Map(_) => quote! { #base_ident.into_iter().map(|(k, v)| (k, v.into())).collect() },
      RustType::Vec(_) => quote! { #base_ident.into_iter().map(Into::into).collect() },
      RustType::Normal(_) => quote! { #base_ident.into() },
      RustType::Boxed(_) => quote! { Box::new((*#base_ident).into()) },
    }
  }

  pub fn from_proto(&self, base_ident: TokenStream2, is_oneof: bool) -> TokenStream2 {
    match &self.rust_type {
      RustType::OptionBoxed(_) => quote! { #base_ident.map(|v| Box::new((*v).into())) },

      _ => self.proto_field.default_from_proto(&base_ident, is_oneof),
    }
  }

  pub fn validator_tokens(&self, validator: &ValidatorExpr) -> TokenStream2 {
    let target_type = self.proto_field.validator_target_type();

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
      RustType::OptionBoxed(_) => quote! { Option<#base_target_type> },
      RustType::Map(_) => base_target_type,
      RustType::Vec(_) => quote! { Vec<#base_target_type> },
      RustType::Normal(_) => base_target_type,
      RustType::Boxed(_) => base_target_type,
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

  pub fn from_type(rust_type: RustType, proto_field: ProtoField, ty: &Type) -> Result<Self, Error> {
    let span = ty.span();

    Ok(Self {
      rust_type,
      span,
      proto_field,
    })
  }

  pub fn error(&self, error: impl Display) -> Error {
    syn::Error::new(self.span, error)
  }
}
