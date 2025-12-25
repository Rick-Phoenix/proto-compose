pub mod builder;
pub use builder::MessageValidatorBuilder;
use builder::state::State;

use proto_types::cel::CelConversionError;

use super::*;
use crate::field_context::ViolationsExt;

impl<T, S: State> ValidatorBuilderFor<T> for MessageValidatorBuilder<T, S>
where
  T: ProtoMessage + PartialEq + Clone + Default + TryInto<::cel::Value, Error = CelConversionError>,
{
  type Target = T;
  type Validator = MessageValidator<T>;

  fn build_validator(self) -> Self::Validator {
    self.build()
  }
}

impl<T> Validator<T> for MessageValidator<T>
where
  T: ProtoMessage + PartialEq + Clone + Default + TryInto<::cel::Value, Error = CelConversionError>,
{
  type Target = T;
  type UniqueStore<'a>
    = LinearRefStore<'a, T>
  where
    Self: 'a;

  impl_testing_methods!();

  fn make_unique_store<'a>(&self, cap: usize) -> Self::UniqueStore<'a> {
    LinearRefStore::default_with_capacity(cap)
  }

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

#[derive(Debug, Clone)]
pub struct MessageValidator<T: ProtoMessage> {
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  pub cel: Vec<&'static CelProgram>,

  pub ignore: Option<Ignore>,

  _message: PhantomData<T>,

  /// Specifies that the field must be set in order to be valid.
  pub required: bool,
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
