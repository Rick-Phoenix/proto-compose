use crate::*;

pub fn generate_oneof_consistency_checks(
  oneof_ident: &Ident,
  variants: &[FieldDataKind],
  no_auto_test: SkipAutoTest,
) -> TokenStream2 {
  let consistency_checks = variants
    .iter()
    .filter_map(|d| d.as_normal())
    .filter_map(|data| data.consistency_check_tokens());

  let auto_test_fn = (!*no_auto_test).then(|| {
    let test_fn_ident = format_ident!(
      "{}_validators_consistency",
      to_snake_case(&oneof_ident.to_string())
    );

    quote! {
      #[cfg(test)]
      #[test]
      fn #test_fn_ident() {
        if let Err(e) = #oneof_ident::check_validators_consistency() {
          panic!("{e}")
        }
      }
    }
  });

  quote! {
    #auto_test_fn

    #[cfg(test)]
    impl #oneof_ident {
      #[track_caller]
      pub fn check_validators_consistency() -> Result<(), ::prelude::OneofErrors> {
        use ::prelude::*;

        let mut field_errors: Vec<::prelude::FieldError> = Vec::new();

        #(#consistency_checks)*

        if field_errors.is_empty() {
          Ok(())
        } else {
          Err(
            ::prelude::OneofErrors {
              oneof_name: stringify!(#oneof_ident),
              field_errors
            }
          )
        }
      }
    }
  }
}

impl OneofCtx<'_> {
  pub fn generate_consistency_checks(&self) -> TokenStream2 {
    generate_oneof_consistency_checks(
      self.proto_enum_ident(),
      &self.variants,
      self.oneof_attrs.no_auto_test,
    )
  }
}
