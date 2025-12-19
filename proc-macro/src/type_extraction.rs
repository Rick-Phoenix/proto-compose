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
    validator_expr: &TokenStream2,
  ) -> TokenStream2 {
    let argument = if is_variant {
      quote! { Some(v) }
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

    let expr = quote! {
      #validator_expr.validate(&#field_context_tokens, parent_elements, #argument).ok_or_push_violations(&mut violations)
    };

    if is_variant {
      quote! { Self::#field_ident(v) => #expr }
    } else {
      expr
    }
  }

  pub fn new(rust_type: TypeInfo, proto_field: &'a ProtoField) -> Result<Self, Error> {
    Ok(Self {
      rust_type,
      proto_field,
    })
  }
}
