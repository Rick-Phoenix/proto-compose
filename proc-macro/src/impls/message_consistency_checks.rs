use crate::*;

pub struct MessageConsistencyChecksCtx<'a> {
  pub item_ident: &'a Ident,
  pub consistency_checks: Vec<TokenStream2>,
  pub no_auto_test: bool,
}

pub fn impl_message_consistency_checks(ctx: MessageConsistencyChecksCtx) -> TokenStream2 {
  let MessageConsistencyChecksCtx {
    item_ident,
    consistency_checks,
    no_auto_test,
  } = ctx;

  let test_module_ident = format_ident!(
    "__{}_consistency_test",
    ccase!(snake, item_ident.to_string())
  );

  let auto_test_fn = if !no_auto_test {
    Some(quote! {
      #[test]
      fn test() {
        if let Err(e) = #item_ident::check_validators_consistency() {
          panic!("{e}")
        }
      }
    })
  } else {
    None
  };

  quote! {
    #[cfg(test)]
    mod #test_module_ident {
      use super::*;

      #auto_test_fn

      impl #item_ident {
        pub(crate) fn check_validators_consistency() -> Result<(), ::prelude::test_utils::MessageTestError> {
          let mut field_errors: Vec<(&'static str, Vec<String>)> = Vec::new();
          let mut cel_errors: Vec<::prelude::CelError> = Vec::new();

          #(
            let (field_name, check) = #consistency_checks;

            if let Err(errs) = check {
              field_errors.push((field_name, errs));
            }
          )*

          let top_level_programs = Self::cel_rules();

          if !top_level_programs.is_empty() {
            if let Err(errs) = ::prelude::test_programs(top_level_programs, Self::default()) {
              cel_errors.extend(errs);
            }
          }

          if !field_errors.is_empty() || !cel_errors.is_empty() {
            return Err(::prelude::test_utils::MessageTestError {
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
