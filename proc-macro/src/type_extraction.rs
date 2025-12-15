use crate::*;

#[derive(Clone)]
pub struct TypeContext {
  pub rust_type: TypeInfo,
  pub proto_field: ProtoField,
}

impl TypeContext {
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

  pub fn field_validator_schema(&self, validator: &CallOrClosure) -> TokenStream2 {
    let target_type = self.proto_field.validator_target_type();

    let validator_expr = match validator {
      CallOrClosure::Call(call) => quote! { #call.build_validator() },

      CallOrClosure::Closure(closure) => {
        quote! { <#target_type as ::prelude::ProtoValidator<#target_type>>::validator_from_closure(#closure) }
      }
    };

    quote! {
      #validator_expr.into_schema()
    }
  }

  pub fn cel_rules_extractor(&self, validator: &CallOrClosure) -> TokenStream2 {
    let target_type = self.proto_field.validator_target_type();

    let validation_expr = match validator {
      CallOrClosure::Call(call) => quote! { #call.build_validator() },

      CallOrClosure::Closure(closure) => {
        quote! { <#target_type as ::prelude::ProtoValidator<#target_type>>::validator_from_closure(#closure) }
      }
    };

    quote! {
      #validation_expr.cel_rules()
    }
  }

  pub fn validator_tokens(
    &self,
    field_ident: &Ident,
    field_context_tokens: TokenStream2,
    validator: &CallOrClosure,
  ) -> TokenStream2 {
    let target_type = self.proto_field.validator_target_type();

    let validation_expr = match validator {
      CallOrClosure::Call(call) => quote! { #call.build_validator() },

      CallOrClosure::Closure(closure) => {
        quote! { <#target_type as ::prelude::ProtoValidator<#target_type>>::validator_from_closure(#closure) }
      }
    };

    let argument = match self.rust_type.type_.as_ref() {
      RustType::Option(inner) => {
        if inner.is_box() {
          quote! { self.#field_ident.as_deref() }
        } else {
          quote! { self.#field_ident.as_ref() }
        }
      }
      RustType::Box(_) => quote! { &(*self.#field_ident) },
      RustType::HashMap(_) => quote! { Some(&self.#field_ident) },
      RustType::Vec(_) => quote! {  Some(&self.#field_ident)  },
      _ => quote! {  Some(&self.#field_ident)  },
    };

    quote! {
      #validation_expr.validate(&#field_context_tokens, parent_elements, #argument).push_violations(&mut violations);
    }
  }

  pub fn from_type(rust_type: TypeInfo, proto_field: ProtoField) -> Result<Self, Error> {
    Ok(Self {
      rust_type,
      proto_field,
    })
  }
}
