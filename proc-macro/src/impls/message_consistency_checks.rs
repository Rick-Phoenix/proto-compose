use std::borrow::Borrow;

use crate::*;

impl<'a, T: Borrow<FieldData>> MessageCtx<'a, T> {
  pub fn generate_consistency_checks(&self) -> TokenStream2 {
    let item_ident = self.proto_struct_ident();

    let consistency_checks = self.non_ignored_fields.iter().filter_map(|data| {
      let FieldData {
        ident_str,
        validator,
        proto_field,
        ..
      } = data.borrow();

      if let ProtoField::Oneof(OneofInfo { path, tags, .. }) = proto_field {
        Some(quote! {
          if let Err(err) = #path::check_tags(#ident_str, &mut [ #(#tags),* ]) {
            field_errors.push(FieldError {
              field: #ident_str,
              errors: vec![err]
            });
          }
        })
      } else {
        validator
          .as_ref()
          // Useless to check consistency for default validators
          .filter(|v| !v.is_fallback)
          .map(|validator| {
            quote! {
              if let Err(errs) = #validator.check_consistency() {
                field_errors.push(FieldError {
                  field: #ident_str,
                  errors: errs
                });
              }
            }
          })
      }
    });

    let auto_test_fn = (!self.message_attrs.no_auto_test).then(|| {
      let test_fn_ident = format_ident!(
        "{}_validators_consistency",
        ccase!(snake, item_ident.to_string())
      );

      quote! {
        #[cfg(test)]
        #[test]
        fn #test_fn_ident() {
          if let Err(e) = #item_ident::check_validators_consistency() {
            panic!("{e}")
          }
        }
      }
    });

    quote! {
      #auto_test_fn

      #[cfg(test)]
      impl #item_ident {
        pub fn check_validators_consistency() -> Result<(), ::prelude::test_utils::MessageTestError> {
          use ::prelude::test_utils::*;

          let mut field_errors: Vec<FieldError> = Vec::new();
          let mut cel_errors: Vec<::prelude::CelError> = Vec::new();

          #(#consistency_checks)*

          let top_level_programs = Self::cel_rules();

          if !top_level_programs.is_empty() {
            if let Err(errs) = ::prelude::test_programs(top_level_programs, Self::default()) {
              cel_errors.extend(errs);
            }
          }

          if !field_errors.is_empty() || !cel_errors.is_empty() {
            return Err(MessageTestError {
                message_full_name: #item_ident::full_name(),
                field_errors,
                cel_errors
              }
            );
          }

          Ok(())
        }
      }
    }
  }
}
