pub mod builder;
pub use builder::MessageValidatorBuilder;
use builder::state::State;

use super::*;

pub trait ValidatedMessage {
  fn validate(&self) -> Result<(), Violations>;

  fn validated(self) -> Result<Self, Violations>
  where
    Self: Sized,
  {
    match self.validate() {
      Ok(()) => Ok(self),
      Err(e) => Err(e),
    }
  }

  fn nested_validate(&self, ctx: &mut ValidationCtx);

  #[must_use]
  fn cel_rules() -> &'static [CelProgram] {
    &[]
  }
}

#[cfg(feature = "cel")]
pub trait TryIntoCel: Clone {
  fn try_into_cel(self) -> Result<::cel::Value, CelError>;
}

#[cfg(feature = "cel")]
impl<E: Display, T: TryInto<::cel::Value, Error = E> + Clone> TryIntoCel for T {
  fn try_into_cel(self) -> Result<::cel::Value, CelError> {
    self
      .try_into()
      .map_err(|e| CelError::ConversionError(e.to_string()))
  }
}

#[cfg(feature = "cel")]
pub trait IntoCelKey: Into<::cel::objects::Key> {}

#[cfg(feature = "cel")]
impl<T> IntoCelKey for T where T: Into<::cel::objects::Key> {}

#[cfg(not(feature = "cel"))]
pub trait IntoCelKey {}
#[cfg(not(feature = "cel"))]
impl<T> IntoCelKey for T {}

#[cfg(not(feature = "cel"))]
pub trait TryIntoCel {}
#[cfg(not(feature = "cel"))]
impl<T> TryIntoCel for T {}

impl<T, S: State> ValidatorBuilderFor<T> for MessageValidatorBuilder<T, S>
where
  T: ProtoMessage + ValidatedMessage + PartialEq + Clone + Default + TryIntoCel,
{
  type Target = T;
  type Validator = MessageValidator<T>;

  fn build_validator(self) -> Self::Validator {
    self.build()
  }
}

impl<T> Validator<T> for MessageValidator<T>
where
  T: ProtoMessage + ValidatedMessage + PartialEq + Clone + Default + TryIntoCel,
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
pub struct MessageValidator<T: ProtoMessage + ValidatedMessage> {
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  pub cel: Vec<CelProgram>,

  pub ignore: Ignore,

  _message: PhantomData<T>,

  /// Specifies that the field must be set in order to be valid.
  pub required: bool,
}

impl<T: ProtoMessage + ValidatedMessage> From<MessageValidator<T>> for ProtoOption {
  fn from(validator: MessageValidator<T>) -> Self {
    let mut rules: OptionValueList = Vec::new();

    insert_cel_rules!(validator, rules);
    insert_boolean_option!(validator, rules, required);

    if !validator.ignore.is_default() {
      rules.push((IGNORE.clone(), validator.ignore.into()))
    }

    Self {
      name: BUF_VALIDATE_FIELD.clone(),
      value: OptionValue::Message(rules.into()),
    }
  }
}
