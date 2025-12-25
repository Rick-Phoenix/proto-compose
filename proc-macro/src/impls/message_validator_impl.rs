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
      fn __validate_internal(&self, field_context: Option<&FieldContext>, parent_elements: &mut Vec<FieldPathElement>) -> Result<(), Violations> {
        let mut violations = Violations::new();

        if let Some(field_context) = field_context {
          parent_elements.push(field_context.as_path_element());
        }

        let top_level_programs = <Self as ::prelude::ProtoMessage>::cel_rules();

        if !top_level_programs.is_empty() {
          let ctx = ProgramsExecutionCtx {
            programs: top_level_programs,
            value: self.clone(),
            violations: &mut violations,
            field_context,
            parent_elements,
          };

          ctx.execute_programs();
        }

        #(#validators_tokens;)*

        if field_context.is_some() {
          parent_elements.pop();
        }

        if violations.is_empty() {
          Ok(())
        } else {
          Err(violations)
        }
      }

      pub fn validate(&self) -> Result<(), Violations> {
        self.__validate_internal(None, &mut vec![])
      }

      pub fn nested_validate(&self, field_context: &FieldContext, parent_elements: &mut Vec<FieldPathElement>) -> Result<(), Violations> {
        self.__validate_internal(Some(field_context), parent_elements)
      }
    }

    impl ::prelude::ProtoValidator for #target_ident {
      type Target = Self;
      type Validator = ::prelude::MessageValidator<Self>;
      type Builder = ::prelude::MessageValidatorBuilder<Self>;

      fn validator_builder() -> Self::Builder {
        ::prelude::MessageValidator::builder()
      }
    }
  }
}
