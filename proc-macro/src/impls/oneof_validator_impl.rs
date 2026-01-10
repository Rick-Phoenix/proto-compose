use crate::*;

pub fn fallback_oneof_validator(oneof_ident: &Ident) -> TokenStream2 {
  quote! {
    impl ::prelude::ValidatedOneof for #oneof_ident {
      fn validate(&self, parent_elements: &mut Vec<::prelude::FieldPathElement>, violations: &mut ::prelude::ViolationsAcc) {
        unreachable!()
      }
    }
  }
}

pub fn generate_oneof_validator<T: Borrow<FieldData>>(
  oneof_ident: &Ident,
  variants: &[T],
) -> TokenStream2 {
  let validators_tokens = variants.iter().filter_map(|data| {
    let data = data.borrow();

    let FieldData {
      ident,
      ident_str,
      tag,
      validator,
      proto_name,
      ..
    } = data;

    if let Some(ValidatorTokens {
      expr: validator_expr,
      span,
      ..
    }) = validator.as_ref()
    {
      let validator_static_ident = format_ident!("{}_VALIDATOR", to_upper_snake_case(ident_str));

      let validator_name = data.validator_name();

      let field_type = data.descriptor_type_tokens();

      Some(quote_spanned! {*span=>
        Self::#ident(v) => {
          static #validator_static_ident: LazyLock<#validator_name> = LazyLock::new(|| {
            #validator_expr
          });

          #validator_static_ident.validate(
            &mut ::prelude::ValidationCtx {
              field_context: ::prelude::FieldContext {
                proto_name: #proto_name,
                tag: #tag,
                field_type: #field_type,
                map_key_type: None,
                map_value_type: None,
                subscript: None,
                field_kind: Default::default(),
              },
              parent_elements,
              violations
            },
            Some(v)
          );
        }
      })
    } else {
      None
    }
  });

  quote! {
    impl ::prelude::ValidatedOneof for #oneof_ident {
      fn validate(&self, parent_elements: &mut Vec<::prelude::FieldPathElement>, violations: &mut ::prelude::ViolationsAcc) {
        match self {
          #(#validators_tokens,)*
          _ => {}
        }
      }
    }
  }
}

impl<T: Borrow<FieldData>> OneofCtx<'_, T> {
  pub fn generate_validator(&self) -> TokenStream2 {
    let oneof_ident = self.proto_enum_ident();
    generate_oneof_validator(oneof_ident, &self.non_ignored_variants)
  }
}
