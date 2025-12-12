use bon::Builder;
use message_validator_builder::{IsUnset, SetIgnore, State};

use super::*;
use crate::field_context::Violations;

impl<T: ProtoMessage, S: State> ValidatorBuilderFor<T> for MessageValidatorBuilder<S> {
  type Target = T;
  type Validator = MessageValidator;

  fn build_validator(self) -> Self::Validator {
    self.build()
  }
}

impl<T: ProtoMessage> Validator<T> for MessageValidator {
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
      parent_elements.push(FieldPathElement {
        field_number: Some(field_context.tag),
        field_name: Some(field_context.name.to_string()),
        field_type: Some(Type::Message as i32),
        key_type: field_context.key_type.map(|t| t as i32),
        value_type: field_context.value_type.map(|t| t as i32),
        subscript: field_context.subscript.clone(),
      });

      val
        .nested_validate(parent_elements)
        .push_violations(violations);

      parent_elements.pop();
    } else {
      violations.add(
        field_context,
        parent_elements,
        &REQUIRED_VIOLATION,
        "is required",
      );
    }

    if violations.is_empty() {
      Ok(())
    } else {
      Err(violations_agg)
    }
  }
}

impl_into_option!(MessageValidator);

#[derive(Debug, Clone, Builder)]
#[builder(derive(Clone))]
pub struct MessageValidator {
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  #[builder(into)]
  pub cel: Option<Arc<[CelRule]>>,
  #[builder(default, with = || true)]
  /// Specifies that the field must be set in order to be valid.
  pub required: bool,
  #[builder(setters(vis = "", name = ignore))]
  pub ignore: Option<Ignore>,
}

impl<S: State> MessageValidatorBuilder<S>
where
  S::Ignore: IsUnset,
{
  /// Rules set for this field will always be ignored.
  pub fn ignore_always(self) -> MessageValidatorBuilder<SetIgnore<S>> {
    self.ignore(Ignore::Always)
  }
}

impl From<MessageValidator> for ProtoOption {
  fn from(validator: MessageValidator) -> Self {
    let mut rules: OptionValueList = Vec::new();

    insert_cel_rules!(validator, rules);
    insert_boolean_option!(validator, rules, required);

    ProtoOption {
      name: BUF_VALIDATE_FIELD.clone(),
      value: OptionValue::Message(rules.into()),
    }
  }
}
