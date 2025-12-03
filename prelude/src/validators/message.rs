use bon::Builder;
use message_validator_builder::{IsUnset, SetIgnore, State};

use super::*;

pub struct GenericMessage;

impl_validator!(MessageValidator, GenericMessage);
impl_into_option!(MessageValidator);

#[derive(Debug, Clone, Builder)]
#[builder(derive(Clone))]
pub struct MessageValidator {
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  #[builder(into)]
  pub cel: Option<Arc<[CelRule]>>,
  #[builder(with = || true)]
  /// Specifies that the field must be set in order to be valid.
  pub required: Option<bool>,
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
