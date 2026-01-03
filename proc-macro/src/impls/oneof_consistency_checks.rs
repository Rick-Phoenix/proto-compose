use crate::*;

impl<'a, T: Borrow<FieldData>> OneofCtx<'a, T> {
  pub fn generate_consistency_checks(&self) -> TokenStream2 {
    let oneof_ident = self.proto_enum_ident();

    let consistency_checks = self
      .non_ignored_variants
      .iter()
      .filter_map(|data| {
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
              if let Err(errs) = #validator.check_consistency() {
                errors.push(FieldError {
                  field: #ident_str,
                  errors: errs
                });
              }
            }
          })
      });

    let auto_test_fn = (!self.oneof_attrs.no_auto_test).then(|| {
      let test_fn_ident = format_ident!(
        "{}_validators_consistency",
        ccase!(snake, oneof_ident.to_string())
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
        pub fn check_validators_consistency() -> Result<(), ::prelude::test_utils::OneofErrors> {
          use ::prelude::test_utils::*;

          let mut errors: Vec<FieldError> = Vec::new();

          #(#consistency_checks)*

          if errors.is_empty() {
            Ok(())
          } else {
            Err(
              OneofErrors {
                oneof_name: stringify!(#oneof_ident),
                field_errors: errors
              }
            )
          }
        }
      }
    }
  }
}
