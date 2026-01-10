use crate::*;

pub fn fallback_message_validator_impl(target_ident: &Ident) -> TokenStream2 {
  quote! {
    impl ::prelude::ValidatedMessage for #target_ident {
      fn validate(&self) -> Result<(), ::prelude::Violations> {
        unimplemented!()
      }

      fn nested_validate(&self, ctx: &mut ::prelude::ValidationCtx) {}
    }

    impl ::prelude::ProtoValidator for #target_ident {
      type Target = Self;
      type Validator = ::prelude::MessageValidator<Self>;
      type Builder = ::prelude::MessageValidatorBuilder<Self>;
    }
  }
}

pub fn generate_message_validator<T: Borrow<FieldData>>(
  target_ident: &Ident,
  fields: &[T],
  top_level_cel_rules: &IterTokensOr<TokenStream2>,
) -> TokenStream2 {
  let validators_tokens = fields.iter().filter_map(|data| {
    let data = data.borrow();

    let FieldData {
      ident,
      type_info,
      ident_str,
      tag,
      validator,
      proto_name,
      proto_field,
      span,
      ..
    } = data;

    if let ProtoField::Oneof(OneofInfo { required, .. }) = proto_field {
      Some(if *required {
        quote_spanned! {*span=>
          match self.#ident.as_ref() {
            Some(oneof) => ::prelude::ValidatedOneof::validate(oneof, parent_elements, violations),
            None => violations.add_required_oneof_violation(parent_elements)
          };
        }
      } else {
        quote_spanned! {*span=>
          if let Some(oneof) = self.#ident.as_ref() {
            ::prelude::ValidatedOneof::validate(oneof, parent_elements, violations);
          }
        }
      })
    } else {
      if let Some(ValidatorTokens {
        expr: validator_expr,
        span,
        ..
      }) = validator.as_ref()
      {
        let validator_static_ident = format_ident!("{}_VALIDATOR", to_upper_snake_case(ident_str));

        let validator_name = data.validator_name();

        let field_type = data.descriptor_type_tokens();

        let argument = {
          match type_info.type_.as_ref() {
            RustType::Option(inner) => {
              if inner.is_box() {
                quote_spanned! (*span=> self.#ident.as_deref())
              } else {
                quote_spanned! (*span=> self.#ident.as_ref())
              }
            }
            RustType::Box(_) => quote_spanned! (*span=> self.#ident.as_deref()),
            _ => {
              if let ProtoField::Single(ProtoType::Message(MessageInfo { .. })) = proto_field {
                quote_spanned! (*span=> self.#ident.as_ref())
              } else {
                quote_spanned! (*span=> Some(&self.#ident))
              }
            }
          }
        };

        Some(quote_spanned! {*span=>
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
      quote_spanned! {top_level_cel_rules.span()=>
        #[inline]
        fn cel_rules() -> &'static [::prelude::CelProgram] {
          static PROGRAMS: std::sync::LazyLock<Vec<::prelude::CelProgram>> = std::sync::LazyLock::new(|| {
            #top_level_cel_rules
          });

          &PROGRAMS
        }
      }
    });

  let cel_rules_call = has_cel_rules.then(|| {
    quote_spanned! {top_level_cel_rules.span()=>
      ::prelude::ValidatedMessage::validate_cel(self, field_context, parent_elements, violations);
    }
  });

  quote! {
    #[allow(clippy::ptr_arg)]
    impl #target_ident {
      #[doc(hidden)]
      fn __validate_internal(&self, field_context: Option<&::prelude::FieldContext>, parent_elements: &mut Vec<::prelude::FieldPathElement>, violations: &mut ::prelude::ViolationsAcc) {
        #cel_rules_call

        #(#validators_tokens)*
      }
    }

    impl ::prelude::ValidatedMessage for #target_ident {
      #cel_rules_method

      fn validate(&self) -> Result<(), ::prelude::Violations> {
        let mut violations = ::prelude::ViolationsAcc::new();

        self.__validate_internal(None, &mut vec![], &mut violations);

        if violations.is_empty() {
          Ok(())
        } else {
          Err(violations.to_vec())
        }
      }

      #[doc(hidden)]
      #[inline]
      fn nested_validate(&self, ctx: &mut ::prelude::ValidationCtx) {
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

impl<T: Borrow<FieldData>> MessageCtx<'_, T> {
  pub fn generate_validator(&self) -> TokenStream2 {
    let target_ident = self.proto_struct_ident();

    generate_message_validator(
      target_ident,
      &self.non_ignored_fields,
      &self.message_attrs.cel_rules,
    )
  }
}
