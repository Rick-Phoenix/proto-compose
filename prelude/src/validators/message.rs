use bon::Builder;
use message_validator_builder::{IsUnset, SetIgnore, State};

use super::*;

pub struct GenericMessage;

impl<S: State> ValidatorBuilderFor<GenericMessage> for MessageValidatorBuilder<S> {}

impl ProtoValidator<GenericMessage> for ValidatorMap {
  type Builder = MessageValidatorBuilder;

  fn builder() -> Self::Builder {
    MessageValidator::builder()
  }
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

/// Used by the [`msg_field`](crate::msg_field) macro to define validation rules.
#[derive(Debug, Clone, Builder)]
pub struct MessageValidator {
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  #[builder(into)]
  pub cel: Option<Arc<[CelRule]>>,
  #[builder(with = || true)]
  /// Marks the field as invalid if unset.
  pub required: Option<bool>,
  #[builder(setters(vis = "", name = ignore))]
  pub ignore: Option<Ignore>,
}

impl<S: message_validator_builder::State> From<MessageValidatorBuilder<S>> for ProtoOption {
  #[track_caller]
  fn from(value: MessageValidatorBuilder<S>) -> Self {
    value.build().into()
  }
}

impl From<MessageValidator> for ProtoOption {
  #[track_caller]
  fn from(validator: MessageValidator) -> Self {
    let mut rules: OptionValueList = Vec::new();

    insert_cel_rules!(validator, rules);
    insert_option!(validator, rules, required);

    ProtoOption {
      name: BUF_VALIDATE_FIELD.clone(),
      value: OptionValue::Message(rules.into()),
    }
  }
}
