mod process_field;
pub use process_field::*;
mod field_or_variant;
pub use field_or_variant::*;

use crate::*;

pub enum ImplKind<'a, 'b> {
  Direct,
  Shadow {
    ignored_fields: &'a mut Vec<Ident>,
    proto_conversion_data: &'a mut ProtoConversionImpl<'b>,
  },
}

pub struct InputItem<'a, 'b> {
  pub impl_kind: ImplKind<'a, 'b>,
  pub validators_tokens: &'b mut Vec<TokenStream2>,
  pub cel_checks_tokens: &'b mut Vec<TokenStream2>,
}

pub fn wrap_with_imports(item_ident: &Ident, tokens: Vec<TokenStream2>) -> TokenStream2 {
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
