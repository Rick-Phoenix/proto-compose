use crate::*;

pub fn impl_message_validator<T>(target_ident: &Ident, fields: &[T]) -> TokenStream2
where
  T: Borrow<FieldData>,
{
  let validators_tokens = fields.iter().flat_map(|data| {
    let FieldData {
      ident,
      type_info,
      ident_str,
      tag,
      validator,
      proto_name,
      proto_field,
      ..
    } = data.borrow();

    if let ProtoField::Oneof { required, .. } = proto_field {
      Some(if *required {
        quote! {
          match self.#ident.as_ref() {
            Some(oneof) => oneof.validate(parent_elements, violations),
            None => violations.add_oneof_required(parent_elements)
          };
        }
      } else {
        quote! {
          if let Some(oneof) = self.#ident.as_ref() {
            oneof.validate(parent_elements, violations);
          }
        }
      })
    } else {
      if let Some(ValidatorTokens {
        expr: validator_expr,
        ..
      }) = validator.as_ref()
      {
        let validator_static_ident = format_ident!("{}_VALIDATOR", ccase!(constant, &ident_str));

        let validator_name = proto_field.validator_name();

        let field_type = proto_field.descriptor_type_tokens();

        let field_context_tokens = quote! {
          ::prelude::FieldContext {
            proto_name: #proto_name,
            tag: #tag,
            field_type: #field_type,
            key_type: None,
            value_type: None,
            subscript: None,
            field_kind: Default::default(),
          }
        };

        let argument = {
          match type_info.type_.as_ref() {
            RustType::Option(inner) => {
              if inner.is_box() {
                quote! { self.#ident.as_deref() }
              } else {
                quote! { self.#ident.as_ref() }
              }
            }
            RustType::Box(_) => quote! { &(*self.#ident) },
            _ => quote! {  Some(&self.#ident)  },
          }
        };

        Some(quote! {
          static #validator_static_ident: LazyLock<#validator_name> = LazyLock::new(|| {
            #validator_expr
          });

          #validator_static_ident.validate(
            &mut ::prelude::ValidationCtx {
              field_context: #field_context_tokens,
              parent_elements,
              violations
            },
            #argument
          );
        })
      } else {
        None
      }
    }
  });

  quote! {
    #[allow(clippy::ptr_arg)]
    impl #target_ident {
      #[doc(hidden)]
      fn __validate_internal(&self, field_context: Option<&FieldContext>, parent_elements: &mut Vec<FieldPathElement>, violations: &mut ViolationsAcc) {
        let top_level_programs = <Self as ::prelude::ProtoMessage>::cel_rules();

        if !top_level_programs.is_empty() {
          let ctx = ProgramsExecutionCtx {
            programs: top_level_programs,
            value: self.clone(),
            violations,
            field_context,
            parent_elements,
          };

          ctx.execute_programs();
        }

        #(#validators_tokens)*
      }

      pub fn validate(&self) -> Result<(), Violations> {
        let mut violations = ViolationsAcc::new();

        self.__validate_internal(None, &mut vec![], &mut violations);

        if violations.is_empty() {
          Ok(())
        } else {
          Err(violations.to_vec())
        }
      }

      pub fn nested_validate(&self, ctx: &mut ValidationCtx) {
        self.__validate_internal(Some(&ctx.field_context), ctx.parent_elements, ctx.violations)
      }
    }

    impl ::prelude::ProtoValidator for #target_ident {
      type Target = Self;
      type Validator = ::prelude::MessageValidator<Self>;
      type Builder = ::prelude::MessageValidatorBuilder<Self>;

      fn default_validator() -> Option<Self::Validator> {
        Some(MessageValidator::default())
      }

      fn validator_builder() -> Self::Builder {
        ::prelude::MessageValidator::builder()
      }
    }
  }
}
