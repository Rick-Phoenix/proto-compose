mod builder;
pub use builder::MessageValidatorBuilder;

use super::*;

pub trait ValidatedMessage: Default {
  fn validate(&self) -> Result<(), Violations>;

  #[inline]
  fn validated(self) -> Result<Self, Violations> {
    match self.validate() {
      Ok(()) => Ok(self),
      Err(e) => Err(e),
    }
  }

  #[doc(hidden)]
  fn nested_validate(&self, ctx: &mut ValidationCtx);

  #[doc(hidden)]
  fn validate_cel(
    &self,
    field_context: Option<&FieldContext>,
    parent_elements: &[FieldPathElement],
    violations: &mut ViolationsAcc,
  ) where
    Self: TryIntoCel,
  {
    let top_level_programs = Self::cel_rules();

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
  }

  #[inline]
  #[must_use]
  #[doc(hidden)]
  fn cel_rules() -> &'static [CelProgram] {
    &[]
  }
}

impl<T, S: builder::State> ValidatorBuilderFor<T> for MessageValidatorBuilder<T, S>
where
  T: ValidatedMessage + PartialEq + TryIntoCel,
{
  type Target = T;
  type Validator = MessageValidator<T>;

  #[inline]
  #[doc(hidden)]
  fn build_validator(self) -> Self::Validator {
    self.build()
  }
}

impl<T> Validator<T> for MessageValidator<T>
where
  T: ValidatedMessage + PartialEq + TryIntoCel,
{
  type Target = T;
  type UniqueStore<'a>
    = LinearRefStore<'a, T>
  where
    Self: 'a;

  impl_testing_methods!();

  fn check_consistency(&self) -> Result<(), Vec<ConsistencyError>> {
    let mut errors = Vec::new();

    #[cfg(feature = "cel")]
    if let Err(e) = self.check_cel_programs() {
      errors.extend(e.into_iter().map(ConsistencyError::from));
    }

    if errors.is_empty() {
      Ok(())
    } else {
      Err(errors)
    }
  }

  #[inline]
  fn make_unique_store<'a>(&self, cap: usize) -> Self::UniqueStore<'a> {
    LinearRefStore::default_with_capacity(cap)
  }

  fn validate(&self, ctx: &mut ValidationCtx, val: Option<&Self::Target>) {
    handle_ignore_always!(&self.ignore);
    handle_ignore_if_zero_value!(&self.ignore, val.is_none());

    if let Some(val) = val {
      ctx
        .parent_elements
        .push(ctx.field_context.as_path_element());

      val.nested_validate(ctx);

      ctx.parent_elements.pop();

      #[cfg(feature = "cel")]
      if !self.cel.is_empty() {
        let ctx = ProgramsExecutionCtx {
          programs: &self.cel,
          value: val.clone(),
          violations: ctx.violations,
          field_context: Some(&ctx.field_context),
          parent_elements: ctx.parent_elements,
        };

        ctx.execute_programs();
      }
    } else if self.required {
      ctx.add_required_violation();
    }
  }
}

#[derive(Debug, Clone, Default)]
pub struct MessageValidator<T: ValidatedMessage> {
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  pub cel: Vec<CelProgram>,

  pub ignore: Ignore,

  _message: PhantomData<T>,

  /// Specifies that the field must be set in order to be valid.
  pub required: bool,
}

impl<T: ValidatedMessage> From<MessageValidator<T>> for ProtoOption {
  fn from(validator: MessageValidator<T>) -> Self {
    let mut rules = OptionMessageBuilder::new();

    rules
      .add_cel_options(validator.cel)
      .set_required(validator.required)
      .set_ignore(validator.ignore);

    Self {
      name: "(buf.validate.field)".into(),
      value: OptionValue::Message(rules.into()),
    }
  }
}
