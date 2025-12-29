use crate::*;

pub struct ValidatorImplCtx<'a> {
  pub target_ident: &'a Ident,
  pub validators_tokens: Vec<TokenStream2>,
}

pub fn impl_message_validator(ctx: ValidatorImplCtx) -> TokenStream2 {
  let ValidatorImplCtx {
    target_ident,
    validators_tokens,
  } = ctx;

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
