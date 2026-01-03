use crate::*;

mod fallback_impls;
pub use fallback_impls::*;

pub fn wrap_with_imports(tokens: Vec<TokenStream2>) -> TokenStream2 {
  quote! {
    const _: () = {
      use std::sync::LazyLock;
      use ::prelude::{*, field_context::ViolationsExt};
      use ::proto_types::{
        protovalidate::{Violations, FieldPathElement},
        field_descriptor_proto::Type,
      };

      #(#tokens)*
    };
  }
}
