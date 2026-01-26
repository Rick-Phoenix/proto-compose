mod builder;
pub use builder::MessageValidatorBuilder;

use super::*;

pub trait ValidatedMessage: ProtoValidation + Default + Clone {
  #[inline]
  fn validate_all(&self) -> Result<(), ViolationsAcc> {
    if !Self::HAS_DEFAULT_VALIDATOR {
      return Ok(());
    }

    let mut ctx = ValidationCtx {
      field_context: None,
      parent_elements: vec![],
      violations: ViolationsAcc::new(),
      fail_fast: false,
    };

    let _ = self.nested_validate(&mut ctx);

    if ctx.violations.is_empty() {
      Ok(())
    } else {
      Err(ctx.violations)
    }
  }

  #[inline]
  fn validate(&self) -> Result<(), ViolationsAcc> {
    if !Self::HAS_DEFAULT_VALIDATOR {
      return Ok(());
    }

    let mut ctx = ValidationCtx::default();

    let _ = self.nested_validate(&mut ctx);

    if ctx.violations.is_empty() {
      Ok(())
    } else {
      Err(ctx.violations)
    }
  }

  #[inline]
  fn is_valid(&self) -> bool {
    if Self::HAS_DEFAULT_VALIDATOR {
      self.validate().is_ok()
    } else {
      true
    }
  }

  #[inline]
  fn validated(self) -> Result<Self, ViolationsAcc> {
    if !Self::HAS_DEFAULT_VALIDATOR {
      return Ok(self);
    }

    match self.validate() {
      Ok(()) => Ok(self),
      Err(e) => Err(e),
    }
  }

  #[doc(hidden)]
  fn nested_validate(&self, ctx: &mut ValidationCtx) -> ValidatorResult;
}

impl<T, S: builder::State> ValidatorBuilderFor<T> for MessageValidatorBuilder<S>
where
  T: ValidatedMessage + PartialEq + TryIntoCel,
{
  type Target = T;
  type Validator = MessageValidator;

  #[inline]
  fn build_validator(self) -> Self::Validator {
    self.build()
  }
}

impl<T> Validator<T> for MessageValidator
where
  T: ValidatedMessage + PartialEq + TryIntoCel,
{
  type Target = T;

  #[cfg(feature = "cel")]
  #[inline(never)]
  #[cold]
  fn check_cel_programs_with(&self, val: Self::Target) -> Result<(), Vec<CelError>> {
    if self.cel.is_empty() {
      Ok(())
    } else {
      test_programs(&self.cel, val)
    }
  }

  #[cfg(feature = "cel")]
  #[inline(never)]
  #[cold]
  fn check_cel_programs(&self) -> Result<(), Vec<CelError>> {
    <Self as Validator<T>>::check_cel_programs_with(self, Self::Target::default())
  }
  #[doc(hidden)]
  fn cel_rules(&self) -> Vec<CelRule> {
    self.cel.iter().map(|p| p.rule.clone()).collect()
  }

  #[inline(never)]
  #[cold]
  fn check_consistency(&self) -> Result<(), Vec<ConsistencyError>> {
    let mut errors = Vec::new();

    #[cfg(feature = "cel")]
    if let Err(e) = <Self as Validator<T>>::check_cel_programs(self) {
      errors.extend(e.into_iter().map(ConsistencyError::from));
    }

    if errors.is_empty() {
      Ok(())
    } else {
      Err(errors)
    }
  }

  fn validate_core<V>(&self, ctx: &mut ValidationCtx, val: Option<&V>) -> ValidatorResult
  where
    V: Borrow<Self::Target> + ?Sized,
  {
    handle_ignore_always!(&self.ignore);
    handle_ignore_if_zero_value!(&self.ignore, val.is_none());

    let mut is_valid = IsValid::Yes;

    if let Some(val) = val {
      let val = val.borrow();

      if let Some(field_context) = &mut ctx.field_context {
        ctx
          .parent_elements
          .push(field_context.as_path_element());
      }

      is_valid &= val.nested_validate(ctx)?;

      if ctx.field_context.is_some() {
        ctx.parent_elements.pop();
      }

      #[cfg(feature = "cel")]
      if !self.cel.is_empty() {
        let cel_ctx = ProgramsExecutionCtx {
          programs: &self.cel,
          value: val.clone(),
          ctx,
        };

        is_valid &= cel_ctx.execute_programs()?;
      }
    } else if self.required {
      is_valid &= ctx.add_required_violation()?;
    }

    Ok(is_valid)
  }

  #[inline(never)]
  #[cold]
  fn schema(&self) -> Option<ValidatorSchema> {
    Some(ValidatorSchema {
      schema: self.clone().into(),
      cel_rules: <Self as Validator<T>>::cel_rules(self),
      imports: vec!["buf/validate/validate.proto".into()],
    })
  }
}

#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default)]
pub struct CelValidator {
  pub programs: Vec<CelProgram>,
}

impl CelValidator {
  #[must_use]
  #[inline]
  pub fn cel(mut self, program: CelProgram) -> Self {
    self.programs.push(program);
    self
  }
}

impl<T> Validator<T> for CelValidator
where
  T: ValidatedMessage + PartialEq + TryIntoCel + Default + Clone,
{
  type Target = T;

  #[cfg(feature = "cel")]
  #[inline(never)]
  #[cold]
  fn check_cel_programs_with(&self, val: Self::Target) -> Result<(), Vec<CelError>> {
    if self.programs.is_empty() {
      Ok(())
    } else {
      test_programs(&self.programs, val)
    }
  }

  #[cfg(feature = "cel")]
  #[inline(never)]
  #[cold]
  fn check_cel_programs(&self) -> Result<(), Vec<CelError>> {
    <Self as Validator<T>>::check_cel_programs_with(self, Self::Target::default())
  }

  #[doc(hidden)]
  fn cel_rules(&self) -> Vec<CelRule> {
    self
      .programs
      .iter()
      .map(|p| p.rule.clone())
      .collect()
  }

  #[inline(never)]
  #[cold]
  fn check_consistency(&self) -> Result<(), Vec<ConsistencyError>> {
    let mut errors = Vec::new();

    #[cfg(feature = "cel")]
    if let Err(e) = <Self as Validator<T>>::check_cel_programs(self) {
      errors.extend(e.into_iter().map(ConsistencyError::from));
    }

    if errors.is_empty() {
      Ok(())
    } else {
      Err(errors)
    }
  }

  fn validate_core<V>(&self, ctx: &mut ValidationCtx, val: Option<&V>) -> ValidatorResult
  where
    V: Borrow<Self::Target> + ?Sized,
  {
    let mut is_valid = IsValid::Yes;

    if let Some(val) = val {
      let val = val.borrow();

      #[cfg(feature = "cel")]
      if !self.programs.is_empty() {
        let cel_ctx = ProgramsExecutionCtx {
          programs: &self.programs,
          value: val.clone(),
          ctx,
        };

        is_valid &= cel_ctx.execute_programs()?;
      }
    }

    Ok(is_valid)
  }

  #[inline(never)]
  #[cold]
  fn schema(&self) -> Option<ValidatorSchema> {
    Some(ValidatorSchema {
      schema: self.clone().into(),
      cel_rules: <Self as Validator<T>>::cel_rules(self),
      imports: vec!["buf/validate/validate.proto".into()],
    })
  }
}

impl From<CelValidator> for ProtoOption {
  #[inline(never)]
  #[cold]
  fn from(value: CelValidator) -> Self {
    let mut rules = OptionMessageBuilder::new();

    rules.add_cel_options(value.programs);

    Self {
      name: "(buf.validate.message)".into(),
      value: OptionValue::Message(rules.into()),
    }
  }
}

#[non_exhaustive]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MessageValidator {
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  pub cel: Vec<CelProgram>,

  pub ignore: Ignore,

  /// Specifies that the field must be set in order to be valid.
  pub required: bool,
}

impl From<MessageValidator> for ProtoOption {
  #[inline(never)]
  #[cold]
  fn from(validator: MessageValidator) -> Self {
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
