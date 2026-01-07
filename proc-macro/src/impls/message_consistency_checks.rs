use std::borrow::Borrow;

use crate::*;

pub fn generate_consistency_checks<T: Borrow<FieldData>>(
  item_ident: &Ident,
  fields: &[T],
  skip_auto_test: bool,
  skip_oneof_tags_check: bool,
) -> TokenStream2 {
  let consistency_checks = fields.iter().filter_map(|data| {
    let FieldData {
      ident_str,
      validator,
      proto_field,
      ..
    } = data.borrow();

    if let ProtoField::Oneof(OneofInfo { path, tags, .. }) = proto_field
      && !skip_oneof_tags_check
    {
      Some(quote! {
        if let Err(err) = <#path as ::prelude::ProtoOneof>::check_tags(#ident_str, &mut [ #(#tags),* ]) {
          field_errors.push(::prelude::FieldError {
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
            if let Err(errs) = ::prelude::Validator::check_consistency(#validator) {
              field_errors.push(::prelude::FieldError {
                field: #ident_str,
                errors: errs
              });
            }
          }
        })
    }
  });

  let auto_test_fn = (!skip_auto_test).then(|| {
    let test_fn_ident = format_ident!(
      "{}_validators_consistency",
      to_snake_case(&item_ident.to_string())
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
      pub fn check_validators_consistency() -> Result<(), ::prelude::MessageTestError> {
        use ::prelude::*;

        let mut field_errors: Vec<::prelude::FieldError> = Vec::new();
        let mut cel_errors: Vec<::prelude::CelError> = Vec::new();

        #(#consistency_checks)*

        let top_level_programs = Self::cel_rules();

        if !top_level_programs.is_empty() {
          if let Err(errs) = ::prelude::test_programs(top_level_programs, Self::default()) {
            cel_errors.extend(errs);
          }
        }

        if !field_errors.is_empty() || !cel_errors.is_empty() {
          return Err(::prelude::MessageTestError {
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

impl<T: Borrow<FieldData>> MessageCtx<'_, T> {
  pub fn generate_consistency_checks(&self) -> TokenStream2 {
    let item_ident = self.proto_struct_ident();

    generate_consistency_checks(
      item_ident,
      &self.non_ignored_fields,
      self.message_attrs.no_auto_test,
      false,
    )
  }
}
