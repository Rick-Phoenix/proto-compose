use bon::Builder;
use message_validator_builder::{IsComplete, IsUnset, SetIgnore, State};
use proto_types::cel::CelConversionError;

use super::*;
use crate::field_context::ViolationsExt;

impl<T, S: State> ValidatorBuilderFor<T> for MessageValidatorBuilder<T, S>
where
  T: ProtoMessage + Clone + Default + TryInto<::cel::Value, Error = CelConversionError>,
{
  type Target = T;
  type Validator = MessageValidator<T>;

  fn build_validator(self) -> Self::Validator {
    self.build()
  }
}

impl<T> Validator<T> for MessageValidator<T>
where
  T: ProtoMessage + Clone + Default + TryInto<::cel::Value, Error = CelConversionError>,
{
  type Target = T;

  impl_testing_methods!();

  fn validate(
    &self,
    field_context: &FieldContext,
    parent_elements: &mut Vec<FieldPathElement>,
    val: Option<&Self::Target>,
  ) -> Result<(), Violations> {
    handle_ignore_always!(&self.ignore);
    handle_ignore_if_zero_value!(&self.ignore, val.is_none());

    let mut violations_agg = Violations::new();
    let violations = &mut violations_agg;

    if let Some(val) = val {
      val
        .nested_validate(field_context, parent_elements)
        .ok_or_push_violations(violations);

      if !self.cel.is_empty() {
        let ctx = ProgramsExecutionCtx {
          programs: &self.cel,
          value: val.clone(),
          violations,
          field_context: Some(field_context),
          parent_elements,
        };

        ctx.execute_programs();
      }
    } else if self.required {
      violations.add_required(field_context, parent_elements);
    }

    if violations.is_empty() {
      Ok(())
    } else {
      Err(violations_agg)
    }
  }
}

#[derive(Debug, Clone, Builder)]
#[builder(derive(Clone))]
pub struct MessageValidator<T: ProtoMessage> {
  #[builder(field)]
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  pub cel: Vec<&'static CelProgram>,

  #[builder(setters(vis = "", name = ignore))]
  pub ignore: Option<Ignore>,

  #[builder(default, setters(vis = ""))]
  _message: PhantomData<T>,

  #[builder(default, with = || true)]
  /// Specifies that the field must be set in order to be valid.
  pub required: bool,
}

impl<T: ProtoMessage, S: State> MessageValidatorBuilder<T, S> {
  /// Adds a custom CEL rule to this validator.
  /// Use the [`cel_program`] or [`inline_cel_program`] macros to build a static program.
  pub fn cel(mut self, program: &'static CelProgram) -> Self {
    self.cel.push(program);
    self
  }

  /// Rules set for this field will always be ignored.
  pub fn ignore_always(self) -> MessageValidatorBuilder<T, SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    self.ignore(Ignore::Always)
  }
}

impl<T: ProtoMessage, S: IsComplete> From<MessageValidatorBuilder<T, S>> for ProtoOption {
  fn from(value: MessageValidatorBuilder<T, S>) -> Self {
    let validator = value.build();
    validator.into()
  }
}

impl<T: ProtoMessage> From<MessageValidator<T>> for ProtoOption {
  fn from(validator: MessageValidator<T>) -> Self {
    let mut rules: OptionValueList = Vec::new();

    insert_cel_rules!(validator, rules);
    insert_boolean_option!(validator, rules, required);

    Self {
      name: BUF_VALIDATE_FIELD.clone(),
      value: OptionValue::Message(rules.into()),
    }
  }
}
