use crate::*;

pub fn generate_oneof_validator(
  use_fallback: UseFallback,
  oneof_ident: &Ident,
  variants: &[FieldDataKind],
) -> TokenStream2 {
  let mut validators_data = ValidatorsData::default();

  let validators_tokens = if *use_fallback {
    quote! { unimplemented!() }
  } else {
    let tokens = variants
      .iter()
      .filter_map(|d| d.as_normal())
      .filter_map(|d| {
        let tokens = field_validator_tokens(oneof_ident, &mut validators_data, d, ItemKind::Oneof);

        (!tokens.is_empty()).then_some((d, tokens))
      })
      .map(|(data, validators)| {
        let ident = &data.ident;

        quote_spanned! {data.span=>
          Self::#ident(v) => {
            #(#validators)*
          }
        }
      });

    quote! { #(#tokens,)* }
  };

  let has_validators = !validators_tokens.is_empty();

  let inline_if_empty = (!has_validators).then(|| quote! { #[inline(always)] });

  let ValidatorsData {
    has_non_default_validators,
    default_check_tokens,
  } = validators_data;

  let has_default_validator_tokens = if has_non_default_validators {
    quote! { true }
    // Means we only encountered boxed self for defaults, which shouldn't happen for oneofs
  } else if default_check_tokens.is_empty() {
    quote! { false }
  } else {
    let mut tokens = TokenStream2::new();

    for (i, expr) in default_check_tokens.into_iter().enumerate() {
      if i != 0 {
        tokens.extend(quote! { || });
      }

      tokens.extend(expr);
    }

    tokens
  };

  quote! {
    impl ::prelude::ValidatedOneof for #oneof_ident {
      #inline_if_empty
      fn validate(&self, ctx: &mut ::prelude::ValidationCtx) -> ::prelude::ValidatorResult {
        let mut is_valid = ::prelude::IsValid::Yes;

        match self {
          #validators_tokens
          _ => {}
        };

        Ok(is_valid)
      }
    }

    impl ::prelude::ProtoValidator for #oneof_ident {
      #[doc(hidden)]
      type Target = Self;
      #[doc(hidden)]
      type Stored = Self;
      #[doc(hidden)]
      type Validator = ::prelude::OneofValidator;
      #[doc(hidden)]
      type Builder = ::prelude::OneofValidator;

      type UniqueStore<'a>
        = ::prelude::LinearRefStore<'a, Self>
      where
        Self: 'a;

      const HAS_DEFAULT_VALIDATOR: bool = #has_default_validator_tokens;
      const HAS_SHALLOW_VALIDATION: bool = #has_non_default_validators;
    }
  }
}

impl OneofCtx<'_> {
  pub fn generate_validator(&self) -> TokenStream2 {
    let oneof_ident = self.proto_enum_ident();

    // For non-reflection implementations we don't skip fields if they don't have
    // validators, so having empty fields means an error occurred
    generate_oneof_validator(
      UseFallback::from(self.variants.is_empty()),
      oneof_ident,
      &self.variants,
    )
  }
}
