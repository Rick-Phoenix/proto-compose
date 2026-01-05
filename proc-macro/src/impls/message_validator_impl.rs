use crate::*;

pub fn fallback_message_validator_impl(target_ident: &Ident) -> TokenStream2 {
  quote! {
    impl ::prelude::ValidatedMessage for #target_ident {
      fn validate(&self) -> Result<(), Violations> {
        unimplemented!()
      }

      fn nested_validate(&self, ctx: &mut ValidationCtx) {}
    }

    impl ::prelude::ProtoValidator for #target_ident {
      type Target = Self;
      type Validator = ::prelude::MessageValidator<Self>;
      type Builder = ::prelude::MessageValidatorBuilder<Self>;

      fn default_validator() -> Option<Self::Validator> {
        unimplemented!()
      }
    }
  }
}

pub fn generate_message_validator(
  target_ident: &Ident,
  fields: &[FieldData],
  top_level_cel_rules: &IterTokensOr<TokenStream2>,
) -> TokenStream2 {
  let validators_tokens = fields.iter().filter_map(|data| {
    let FieldData {
      ident,
      type_info,
      ident_str,
      tag,
      validator,
      proto_name,
      proto_field,
      ..
    } = data;

    if let ProtoField::Oneof(OneofInfo { required, .. }) = proto_field {
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
              field_context: ::prelude::FieldContext {
                proto_name: #proto_name,
                tag: #tag,
                field_type: #field_type,
                key_type: None,
                value_type: None,
                subscript: None,
                field_kind: Default::default(),
              },
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

  let has_cel_rules = !top_level_cel_rules.is_empty();

  let cel_rules_method = has_cel_rules.then(|| {
      quote! {
        #[inline]
        fn cel_rules() -> &'static [CelProgram] {
          static PROGRAMS: std::sync::LazyLock<Vec<::prelude::CelProgram>> = std::sync::LazyLock::new(|| {
            #top_level_cel_rules
          });

          &PROGRAMS
        }
      }
    });

  let cel_rules_call = has_cel_rules.then(|| {
    quote! {
      ::prelude::ValidatedMessage::validate_cel(self, field_context, parent_elements, violations);
    }
  });

  quote! {
    #[allow(clippy::ptr_arg)]
    impl #target_ident {
      #[doc(hidden)]
      fn __validate_internal(&self, field_context: Option<&FieldContext>, parent_elements: &mut Vec<FieldPathElement>, violations: &mut ViolationsAcc) {
        #cel_rules_call

        #(#validators_tokens)*
      }
    }

    impl ::prelude::ValidatedMessage for #target_ident {
      #cel_rules_method

      fn validate(&self) -> Result<(), Violations> {
        let mut violations = ViolationsAcc::new();

        self.__validate_internal(None, &mut vec![], &mut violations);

        if violations.is_empty() {
          Ok(())
        } else {
          Err(violations.to_vec())
        }
      }

      #[doc(hidden)]
      #[inline]
      fn nested_validate(&self, ctx: &mut ValidationCtx) {
        self.__validate_internal(Some(&ctx.field_context), ctx.parent_elements, ctx.violations)
      }
    }

    impl ::prelude::ProtoValidator for #target_ident {
      #[doc(hidden)]
      type Target = Self;
      #[doc(hidden)]
      type Validator = ::prelude::MessageValidator<Self>;
      #[doc(hidden)]
      type Builder = ::prelude::MessageValidatorBuilder<Self>;

      #[doc(hidden)]
      #[inline]
      fn default_validator() -> Option<Self::Validator> {
        Some(MessageValidator::default())
      }
    }
  }
}

impl<'a, T: Borrow<FieldData>> MessageCtx<'a, T> {
  pub fn generate_validator(&self) -> TokenStream2 {
    let target_ident = self.proto_struct_ident();

    let validators_tokens = self.non_ignored_fields.iter().filter_map(|data| {
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

      if let ProtoField::Oneof(OneofInfo { required, .. }) = proto_field {
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
                field_context: ::prelude::FieldContext {
                  proto_name: #proto_name,
                  tag: #tag,
                  field_type: #field_type,
                  key_type: None,
                  value_type: None,
                  subscript: None,
                  field_kind: Default::default(),
                },
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

    let top_level_cel_rules = &self.message_attrs.cel_rules;

    let has_cel_rules = !top_level_cel_rules.is_empty();

    let cel_rules_method = has_cel_rules.then(|| {
      quote! {
        #[inline]
        fn cel_rules() -> &'static [CelProgram] {
          static PROGRAMS: std::sync::LazyLock<Vec<::prelude::CelProgram>> = std::sync::LazyLock::new(|| {
            #top_level_cel_rules
          });

          &PROGRAMS
        }
      }
    });

    let cel_rules_call = has_cel_rules.then(|| {
      quote! {
        ::prelude::ValidatedMessage::validate_cel(self, field_context, parent_elements, violations);
      }
    });

    quote! {
      #[allow(clippy::ptr_arg)]
      impl #target_ident {
        #[doc(hidden)]
        fn __validate_internal(&self, field_context: Option<&FieldContext>, parent_elements: &mut Vec<FieldPathElement>, violations: &mut ViolationsAcc) {
          #cel_rules_call

          #(#validators_tokens)*
        }
      }

      impl ::prelude::ValidatedMessage for #target_ident {
        #cel_rules_method

        fn validate(&self) -> Result<(), Violations> {
          let mut violations = ViolationsAcc::new();

          self.__validate_internal(None, &mut vec![], &mut violations);

          if violations.is_empty() {
            Ok(())
          } else {
            Err(violations.to_vec())
          }
        }

        #[doc(hidden)]
        #[inline]
        fn nested_validate(&self, ctx: &mut ValidationCtx) {
          self.__validate_internal(Some(&ctx.field_context), ctx.parent_elements, ctx.violations)
        }
      }

      impl ::prelude::ProtoValidator for #target_ident {
        #[doc(hidden)]
        type Target = Self;
        #[doc(hidden)]
        type Validator = ::prelude::MessageValidator<Self>;
        #[doc(hidden)]
        type Builder = ::prelude::MessageValidatorBuilder<Self>;

        #[doc(hidden)]
        #[inline]
        fn default_validator() -> Option<Self::Validator> {
          Some(MessageValidator::default())
        }
      }
    }
  }
}
