use bon::Builder;
use bool_validator_builder::State;

use super::*;

impl_validator!(BoolValidator, bool);
impl_into_option!(BoolValidator);

#[derive(Clone, Debug, Builder)]
pub struct BoolValidator {
  /// Specifies that only this specific value will be considered valid for this field.
  pub const_: Option<bool>,
  #[builder(with = || true)]
  /// Specifies that the field must be set in order to be valid.
  pub required: Option<bool>,
}

impl From<BoolValidator> for ProtoOption {
  fn from(validator: BoolValidator) -> Self {
    let mut rules: OptionValueList = Vec::new();

    insert_option!(validator, rules, const_);

    let mut outer_rules: OptionValueList = vec![];

    outer_rules.push((BOOL.clone(), OptionValue::Message(rules.into())));

    insert_option!(validator, outer_rules, required);

    ProtoOption {
      name: BUF_VALIDATE_FIELD.clone(),
      value: OptionValue::Message(outer_rules.into()),
    }
  }
}
