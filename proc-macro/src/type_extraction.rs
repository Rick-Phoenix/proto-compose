use crate::*;

pub fn generate_validator_tokens(
  rust_type: &RustType,
  is_variant: bool,
  field_ident: &Ident,
  field_context_tokens: TokenStream2,
  validator_static_ident: &Ident,
  validator_static_tokens: TokenStream2,
) -> TokenStream2 {
  let argument = if is_variant {
    quote! { Some(v) }
  } else {
    match rust_type {
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

  let validator_impl = quote! {
    #validator_static_tokens

    #validator_static_ident.validate(&#field_context_tokens, parent_elements, #argument).ok_or_push_violations(&mut violations);
  };

  if is_variant {
    quote! {
      Self::#field_ident(v) => {
        #validator_impl
      }
    }
  } else {
    validator_impl
  }
}
