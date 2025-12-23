use crate::*;

pub struct MessageCelChecksCtx<'a> {
  pub item_ident: &'a Ident,
  pub field_cel_checks: Vec<TokenStream2>,
  pub no_auto_test: bool,
  pub message_name: &'a str,
}

pub fn impl_message_cel_checks(ctx: MessageCelChecksCtx) -> TokenStream2 {
  let MessageCelChecksCtx {
    item_ident,
    field_cel_checks,
    no_auto_test,
    message_name,
  } = ctx;

  let test_module_ident = format_ident!("__{}_cel_test", ccase!(snake, item_ident.to_string()));

  let auto_test_fn = if !no_auto_test {
    Some(quote! {
      #[test]
      fn test() {
        #item_ident::check_cel_programs()
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
        #[track_caller]
        pub(crate) fn check_cel_programs() {
          let mut errors: Vec<::prelude::CelError> = Vec::new();

          #(
            if let Err(errs) = #field_cel_checks {
              errors.extend(errs);
            }
          )*

          let top_level_programs = Self::cel_rules();

          if !top_level_programs.is_empty() {
            if let Err(errs) = ::prelude::test_programs(top_level_programs, Self::default()) {
              errors.extend(errs);
            }
          }

          if !errors.is_empty() {
            let err = ::prelude::test_utils::cel_programs_error(#message_name, errors);

            panic!("{err}")
          }
        }
      }
    }
  }
}
