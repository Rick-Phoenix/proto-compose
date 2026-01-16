use crate::*;

pub fn generate_oneof_validator(
  use_fallback: UseFallback,
  oneof_ident: &Ident,
  variants: &[FieldDataKind],
) -> TokenStream2 {
  let validators_tokens = if *use_fallback {
    quote! { unimplemented!() }
  } else {
    let tokens = variants
      .iter()
      .filter_map(|d| d.as_normal())
      .filter_map(|data| {
        field_validator_tokens(data, ItemKind::Oneof).map(|inner| {
          let ident = &data.ident;

          quote_spanned! {data.span=>
            Self::#ident(v) => {
              #inner
            }
          }
        })
      });

    quote! {
      match self {
        #(#tokens,)*
        _ => true
      }
    }
  };

  // Validators will always be populated if at least one field
  // is a message, because we cannot know if it has validators
  // of its own
  if validators_tokens.is_empty() {
    quote! {
      impl ::prelude::ValidatedOneof for #oneof_ident {
        #[inline(always)]
        fn validate(&self, _: &mut ::prelude::ValidationCtx) -> bool {
          true
        }
      }
    }
  } else {
    quote! {
      impl ::prelude::ValidatedOneof for #oneof_ident {
        fn validate(&self, ctx: &mut ::prelude::ValidationCtx) -> bool {
          #validators_tokens
        }
      }
    }
  }
}

impl OneofCtx<'_> {
  pub fn generate_validator(&self) -> TokenStream2 {
    let oneof_ident = self.proto_enum_ident();

    // For non-reflection implementations we don't skip fields if they don't have
    // validators, so empty fields = an error occurred
    generate_oneof_validator(self.variants.is_empty().into(), oneof_ident, &self.variants)
  }
}
