use ::proto_types::protovalidate::StringRules;

use crate::*;

pub fn get_string_validator(rules: &StringRules) -> TokenStream2 {
  let mut validator = quote! { ::prelude::StringValidator::builder() };

  if let Some(min_len) = rules.min_len {
    let min_len = min_len as usize;

    validator.extend(quote! { .min_len(#min_len) });
  }

  quote! { #validator.build() }
}
