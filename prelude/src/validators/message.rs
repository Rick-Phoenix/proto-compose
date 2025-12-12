use bon::Builder;
use message_validator_builder::{IsComplete, IsUnset, SetIgnore, State};

use super::*;
use crate::field_context::Violations;

impl<T: ProtoMessage, S: State> ValidatorBuilderFor<T> for MessageValidatorBuilder<T, S> {
  type Target = T;
  type Validator = MessageValidator<T>;

  fn build_validator(self) -> Self::Validator {
    self.build()
  }
}

impl<T: ProtoMessage> Validator<T> for MessageValidator<T> {
  type Target = T;

  fn validate(
    &self,
    field_context: &FieldContext,
    parent_elements: &mut Vec<FieldPathElement>,
    val: Option<&Self::Target>,
  ) -> Result<(), Vec<Violation>> {
    let mut violations_agg: Vec<Violation> = Vec::new();
    let violations = &mut violations_agg;

    if let Some(val) = val {
      val
        .nested_validate(field_context, parent_elements)
        .push_violations(violations);
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
  #[builder(default, setters(vis = ""))]
  _message: PhantomData<T>,
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  #[builder(into)]
  pub cel: Option<Arc<[CelRule]>>,
  #[builder(default, with = || true)]
  /// Specifies that the field must be set in order to be valid.
  pub required: bool,
  #[builder(setters(vis = "", name = ignore))]
  pub ignore: Option<Ignore>,
}

impl<T: ProtoMessage, S: State> MessageValidatorBuilder<T, S>
where
  S::Ignore: IsUnset,
{
  /// Rules set for this field will always be ignored.
  pub fn ignore_always(self) -> MessageValidatorBuilder<T, SetIgnore<S>> {
    self.ignore(Ignore::Always)
  }
}

impl<T: ProtoMessage, S: IsComplete> From<MessageValidatorBuilder<T, S>> for ProtoOption {
  fn from(value: MessageValidatorBuilder<T, S>) -> ProtoOption {
    let validator = value.build();
    validator.into()
  }
}

impl<T: ProtoMessage> From<MessageValidator<T>> for ProtoOption {
  fn from(validator: MessageValidator<T>) -> Self {
    let mut rules: OptionValueList = Vec::new();

    insert_cel_rules!(validator, rules);
    insert_boolean_option!(validator, rules, required);

    ProtoOption {
      name: BUF_VALIDATE_FIELD.clone(),
      value: OptionValue::Message(rules.into()),
    }
  }
}
