use crate::*;

pub fn fallback_oneof_validator(oneof_ident: &Ident) -> TokenStream2 {
  quote! {
    impl ::prelude::ValidatedOneof for #oneof_ident {
      fn validate(&self, parent_elements: &mut Vec<FieldPathElement>, violations: &mut ViolationsAcc) {
        unreachable!()
      }
    }
  }
}

pub fn generate_oneof_validator(oneof_ident: &Ident, variants: &[FieldData]) -> TokenStream2 {
  let validators_tokens = variants.iter().filter_map(|data| {
    let FieldData {
      ident,
      ident_str,
      tag,
      validator,
      proto_name,
      proto_field,
      ..
    } = data;

    if let Some(ValidatorTokens {
      expr: validator_expr,
      ..
    }) = validator.as_ref()
    {
      let validator_static_ident = format_ident!("{}_VALIDATOR", to_upper_snake_case(ident_str));

      let validator_name = proto_field.validator_name();

      let field_type = proto_field.descriptor_type_tokens();

      Some(quote! {
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
      fn validate(&self, parent_elements: &mut Vec<FieldPathElement>, violations: &mut ViolationsAcc) {
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

    let validators_tokens = self
      .non_ignored_variants
      .iter()
      .filter_map(|data| {
        let FieldData {
          ident,
          ident_str,
          tag,
          validator,
          proto_name,
          proto_field,
          ..
        } = data.borrow();

        if let Some(ValidatorTokens {
          expr: validator_expr,
          ..
        }) = validator.as_ref()
        {
          let validator_static_ident =
            format_ident!("{}_VALIDATOR", to_upper_snake_case(ident_str));

          let validator_name = proto_field.validator_name();

          let field_type = proto_field.descriptor_type_tokens();

          Some(quote! {
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
        fn validate(&self, parent_elements: &mut Vec<FieldPathElement>, violations: &mut ViolationsAcc) {
          match self {
            #(#validators_tokens,)*
            _ => {}
          }
        }
      }
    }
  }
}
