use crate::*;

pub fn generate_consistency_checks<T: Borrow<FieldData>>(
  oneof_ident: &Ident,
  variants: &[T],
  no_auto_test: bool,
) -> TokenStream2 {
  let consistency_checks = variants.iter().filter_map(|data| {
    let FieldData {
      ident_str,
      validator,
      ..
    } = data.borrow();

    validator
      .as_ref()
      // Useless to check consistency for default validators
      .filter(|v| !v.is_fallback)
      .map(|validator| {
        quote! {
          if let Err(errs) = ::prelude::Validator::check_consistency(#validator) {
            errors.push(::prelude::FieldError {
              field: #ident_str,
              errors: errs
            });
          }
        }
      })
  });

  let auto_test_fn = (!no_auto_test).then(|| {
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
      pub fn check_validators_consistency() -> Result<(), ::prelude::OneofErrors> {
        use ::prelude::*;

        let mut errors: Vec<::prelude::FieldError> = Vec::new();

        #(#consistency_checks)*

        if errors.is_empty() {
          Ok(())
        } else {
          Err(
            ::prelude::OneofErrors {
              oneof_name: stringify!(#oneof_ident),
              field_errors: errors
            }
          )
        }
      }
    }
  }
}

impl<T: Borrow<FieldData>> OneofCtx<'_, T> {
  pub fn generate_consistency_checks(&self) -> TokenStream2 {
    generate_consistency_checks(
      self.proto_enum_ident(),
      &self.non_ignored_variants,
      self.oneof_attrs.no_auto_test,
    )
  }
}
