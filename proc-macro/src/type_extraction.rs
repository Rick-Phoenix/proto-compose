use crate::*;

#[derive(Clone)]
pub struct TypeInfo {
  pub rust_type: RustType,
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

  pub fn field_into_proto_impl(&self, base_ident: TokenStream2) -> TokenStream2 {
    self.proto_field.default_into_proto(&base_ident)
  }

  pub fn field_from_proto_impl(&self, base_ident: TokenStream2) -> TokenStream2 {
    self.proto_field.default_from_proto(&base_ident)
  }

  pub fn validator_schema_tokens(&self, validator: &ValidatorExpr) -> TokenStream2 {
    let target_type = self.proto_field.validator_target_type();

    match validator {
      ValidatorExpr::Call(call) => {
        quote! { Some(<#target_type as ::prelude::ProtoValidator<#target_type>>::from_builder(#call)) }
      }

      ValidatorExpr::Closure(closure) => {
        quote! { Some(<#target_type as ::prelude::ProtoValidator<#target_type>>::build_rules(#closure)) }
      }
    }
  }

  pub fn validator_tokens(&self, validator: &ValidatorExpr) -> TokenStream2 {
    let target_type = self.proto_field.validator_target_type();

    match validator {
      ValidatorExpr::Call(call) => quote! { #call.build_validator() },

      ValidatorExpr::Closure(closure) => {
        quote! { <#target_type as ::prelude::ProtoValidator<#target_type>>::validator_from_closure(#closure) }
      }
    }
  }

  pub fn from_type(rust_type: RustType, proto_field: ProtoField) -> Result<Self, Error> {
    Ok(Self {
      rust_type,
      proto_field,
    })
  }
}
