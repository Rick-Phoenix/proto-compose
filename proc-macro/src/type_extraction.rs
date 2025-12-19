use crate::*;

#[derive(Clone)]
pub struct TypeContext<'a> {
  pub rust_type: TypeInfo,
  pub proto_field: &'a ProtoField,
}

impl<'a> TypeContext<'a> {
  pub fn validator_tokens(
    &self,
    is_variant: bool,
    field_ident: &Ident,
    field_context_tokens: TokenStream2,
    validator: &FieldValidatorExpr,
  ) -> TokenStream2 {
    let validator_expr = validator.build_expr();

    let argument = if is_variant {
      quote! { v }
    } else {
      match self.rust_type.type_.as_ref() {
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
      }
    };

    quote! {
      #validator_expr.validate(&#field_context_tokens, parent_elements, #argument).ok_or_push_violations(&mut violations);
    }
  }

  pub fn new(rust_type: TypeInfo, proto_field: &'a ProtoField) -> Result<Self, Error> {
    Ok(Self {
      rust_type,
      proto_field,
    })
  }
}

pub struct FieldValidatorExpr<'a> {
  pub target_type: TokenStream2,
  pub definition_expr: &'a CallOrClosure,
}

impl<'a> FieldValidatorExpr<'a> {
  pub fn new(proto_field: &ProtoField, definition_expr: &'a CallOrClosure) -> Self {
    Self {
      target_type: proto_field.validator_target_type(),
      definition_expr,
    }
  }

  pub fn build_expr(&self) -> TokenStream2 {
    let Self {
      target_type,
      definition_expr,
    } = self;

    match definition_expr {
      CallOrClosure::Call(call) => quote! { #call.build_validator() },

      CallOrClosure::Closure(closure) => {
        quote! { <#target_type as ::prelude::ProtoValidator>::validator_from_closure(#closure) }
      }
    }
  }

  pub fn schema_expr(&self) -> TokenStream2 {
    let validator_expr = self.build_expr();

    quote! {
      #validator_expr.into_schema()
    }
  }

  pub fn cel_check_expr(&self) -> TokenStream2 {
    let validator_expr = self.build_expr();

    quote! {
      #validator_expr.check_cel_programs()
    }
  }
}
