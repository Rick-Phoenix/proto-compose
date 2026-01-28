use crate::*;

pub fn generate_oneofs_tags_checks(
  item_ident: &Ident,
  fields_data: &[FieldDataKind],
  auto_tests: AutoTests,
  message_name: &str,
) -> TokenStream2 {
  let oneofs_tags_checks = fields_data
    .iter()
    .filter_map(|d| d.as_normal())
    .filter_map(|data| {
      let FieldData {
        proto_field, span, ..
      } = data;

      if let ProtoField::Oneof(OneofInfo { path, tags, .. }) = proto_field {
        Some(quote_spanned! {*span=>
          <#path as ::prelude::ProtoOneof>::check_tags(#message_name, &mut [ #(#tags),* ])?;
        })
      } else {
        None
      }
    });

  let oneofs_auto_test = (!auto_tests.skip_oneof_tags_check).then(|| {
    let test_fn_ident = format_ident!("{}_oneofs_tags", to_snake_case(&item_ident.to_string()));

    quote! {
      #[cfg(test)]
      #[test]
      fn #test_fn_ident() {
        if let Err(e) = #item_ident::check_oneofs_tags() {
          panic!("{e}")
        }
      }
    }
  });

  quote! {
    #oneofs_auto_test

    #[cfg(test)]
    impl #item_ident {
      #[allow(unused)]
      #[track_caller]
      pub fn check_oneofs_tags() -> Result<(), String> {
        #(#oneofs_tags_checks)*

        Ok(())
      }
    }
  }
}

pub fn generate_message_validators_consistency_checks(
  item_ident: &Ident,
  fields_data: &[FieldDataKind],
  auto_tests: AutoTests,
  message_name: &str,
  top_level_validators: &Validators,
) -> TokenStream2 {
  let consistency_checks = fields_data
    .iter()
    .filter_map(|d| d.as_normal())
    .filter_map(|data| data.consistency_check_tokens())
    .chain(top_level_validators.iter().map(|v| {
      quote! {
        if let Err(errs) = ::prelude::Validator::<#item_ident>::check_consistency(&#v) {
          top_level_errors.extend(errs);
        }
      }
    }));

  let validators_auto_test_fn = (!auto_tests.skip_consistency_checks).then(|| {
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
    #validators_auto_test_fn

    #[cfg(test)]
    impl #item_ident {
      #[allow(unused)]
      #[track_caller]
      pub fn check_validators_consistency() -> Result<(), ::prelude::MessageTestError> {
        let mut field_errors: Vec<::prelude::FieldError> = Vec::new();
        let mut top_level_errors: Vec<::prelude::ConsistencyError> = Vec::new();

        #(#consistency_checks)*

        if !field_errors.is_empty() || !top_level_errors.is_empty() {
          return Err(::prelude::MessageTestError {
              message_full_name: #message_name,
              field_errors,
              top_level_errors
            }
          );
        }

        Ok(())
      }
    }
  }
}

impl MessageCtx<'_> {
  pub fn generate_consistency_checks(&self) -> TokenStream2 {
    let item_ident = self.proto_struct_ident();

    let validators_checks = generate_message_validators_consistency_checks(
      item_ident,
      &self.fields_data,
      self.message_attrs.auto_tests,
      &self.message_attrs.name,
      &self.message_attrs.validators,
    );

    let oneofs_checks = generate_oneofs_tags_checks(
      item_ident,
      &self.fields_data,
      self.message_attrs.auto_tests,
      &self.message_attrs.name,
    );

    quote! {
      #validators_checks
      #oneofs_checks
    }
  }
}
