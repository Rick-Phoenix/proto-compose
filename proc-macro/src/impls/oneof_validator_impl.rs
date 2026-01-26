use crate::*;

pub fn generate_oneof_validator(
  use_fallback: UseFallback,
  oneof_ident: &Ident,
  variants: &[FieldDataKind],
  top_level_validators: &Validators,
) -> TokenStream2 {
  let mut validators_data = ValidatorsData {
    has_non_default_validators: !top_level_validators.is_empty(),
    default_check_tokens: Vec::new(),
  };

  let validators_tokens = if *use_fallback {
    quote! { unimplemented!() }
  } else {
    let top_level = top_level_validators.iter().map(|v| {
      quote_spanned! {v.span=>
        is_valid &= ::prelude::Validator::<#oneof_ident>::validate_core(
          &(#v),
          ctx,
          Some(self)
        )?;
      }
    });

    let variants_validators = variants
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

    let top_level_tokens = quote! { #(#top_level)* };
    let variants_tokens = quote! { #(#variants_validators,)* };

    if top_level_tokens.is_empty() && variants_tokens.is_empty() {
      TokenStream2::new()
    } else {
      quote! {
        #top_level_tokens

        match self {
          #variants_tokens
          _ => {}
        };
      }
    }
  };

  let has_validators = !validators_tokens.is_empty();

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

  let inline_if_empty = (!has_validators).then(|| quote! { #[inline(always)] });

  quote! {
    impl ::prelude::ValidatedOneof for #oneof_ident {
      #inline_if_empty
      fn validate(&self, ctx: &mut ::prelude::ValidationCtx) -> ::prelude::ValidationResult {
        let mut is_valid = ::prelude::IsValid::Yes;

        #validators_tokens

        Ok(is_valid)
      }
    }

    impl ::prelude::ProtoValidation for #oneof_ident {
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
      #[doc(hidden)]
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
      &self.oneof_attrs.validators,
    )
  }
}
