use std::{borrow::Cow, fmt::Display};

use syn::spanned::Spanned;

use crate::*;

#[derive(Clone)]
pub struct TypeInfo {
  pub rust_type: RustType,
  pub span: Span,
}

impl TypeInfo {
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
      RustType::Boxed(_) => quote! { Option<#base_target_type> },
      RustType::Map(_) => base_target_type,
      RustType::Vec(_) => quote! { Vec<#base_target_type> },
      RustType::Normal(_) => base_target_type,
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

  pub fn from_type(ty: &Type) -> Result<Self, Error> {
    let path = extract_type_path(ty)?;
    let rust_type = RustType::from_path(path);

    let span = ty.span();

    Ok(Self { rust_type, span })
  }

  pub fn error(&self, error: impl Display) -> Error {
    syn::Error::new(self.span, error)
  }
}
