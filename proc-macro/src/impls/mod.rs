use crate::*;

mod fallback_impls;
pub use fallback_impls::*;
mod conversions;
pub use oneof_validator_impl::*;
mod oneof_validator_impl;
pub use conversions::*;
mod message_consistency_checks;
pub use message_consistency_checks::*;
mod message_validator_impl;
pub use message_validator_impl::*;
mod oneof_consistency_checks;
pub use oneof_consistency_checks::*;

pub const fn is_no_std() -> bool {
  cfg!(not(feature = "std"))
}

pub fn guard_inventory_on_no_std() -> Option<TokenStream2> {
  is_no_std().then(|| quote! { #[cfg(feature = "std")] })
}

pub fn wrap_with_imports(tokens: &TokenStream2) -> TokenStream2 {
  quote! {
    const _: () = {
      use ::prelude::*;
      use ::prelude::alloc::{vec, vec::Vec, format, string::String};
      use ::prelude::proto_types::{
        protovalidate::{Violations, FieldPathElement},
        field_descriptor_proto::Type,
      };

      #tokens
    };
  }
}

pub fn wrap_multiple_with_imports(tokens: &[TokenStream2]) -> TokenStream2 {
  wrap_with_imports(&quote! { #(#tokens)* })
}

pub fn options_tokens(
  span: Span,
  options: &TokensOr<TokenStream2>,
  deprecated: bool,
) -> TokenStream2 {
  if deprecated {
    quote_spanned! {span=>
      {
        let mut options: Vec<::prelude::ProtoOption> = #options;
        options.push(::prelude::proto_deprecated());
        options
      }
    }
  } else {
    options.to_token_stream()
  }
}
