use bon::Builder;
use proto_types::{Duration, Timestamp};
use timestamp_validator_builder::{IsUnset, SetIgnore, State};

use super::*;

impl_validator!(TimestampValidator, Timestamp);
impl_into_option!(TimestampValidator);

#[derive(Clone, Debug, Builder)]
#[builder(derive(Clone))]
pub struct TimestampValidator {
  /// Specifies that only this specific value will be considered valid for this field.
  pub const_: Option<Timestamp>,
  /// Specifies that this field's value will be valid only if it is smaller than the specified amount.
  pub lt: Option<Timestamp>,
  /// Specifies that this field's value will be valid only if it is smaller than, or equal to, the specified amount.
  pub lte: Option<Timestamp>,
  #[builder(with = || true)]
  /// Specifies that this field's value will be valid only if it in the past.
  pub lt_now: Option<bool>,
  /// Specifies that this field's value will be valid only if it is greater than the specified amount.
  pub gt: Option<Timestamp>,
  /// Specifies that this field's value will be valid only if it is greater than, or equal to, the specified amount.
  pub gte: Option<Timestamp>,
  #[builder(with = || true)]
  /// Specifies that this field's value will be valid only if it in the future.
  pub gt_now: Option<bool>,
  /// Specifies that this field's value will be valid only if it is within the specified Duration (either in the past or future) from the moment when it's being validated.
  pub within: Option<Duration>,
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  #[builder(into)]
  pub cel: Option<Box<[CelRule]>>,
  #[builder(with = || true)]
  /// Specifies that the field must be set in order to be valid.
  pub required: Option<bool>,
  #[builder(setters(vis = "", name = ignore))]
  pub ignore: Option<Ignore>,
}

impl<S: State> TimestampValidatorBuilder<S>
where
  S::Ignore: IsUnset,
{
  /// Rules set for this field will always be ignored.
  pub fn ignore_always(self) -> TimestampValidatorBuilder<SetIgnore<S>> {
    self.ignore(Ignore::Always)
  }
}

impl From<TimestampValidator> for ProtoOption {
  fn from(validator: TimestampValidator) -> Self {
    let mut rules: OptionValueList = Vec::new();

    if let Some(const_val) = validator.const_ {
      rules.push((CONST_.clone(), OptionValue::Timestamp(const_val)));
    }

    insert_option!(validator, rules, lt);
    insert_option!(validator, rules, lte);
    insert_option!(validator, rules, gt);
    insert_option!(validator, rules, gte);
    insert_option!(validator, rules, lt_now);
    insert_option!(validator, rules, gt_now);
    insert_option!(validator, rules, within);

    let mut outer_rules: OptionValueList = vec![];

    outer_rules.push((TIMESTAMP.clone(), OptionValue::Message(rules.into())));

    insert_cel_rules!(validator, outer_rules);
    insert_option!(validator, outer_rules, required);
    insert_option!(validator, outer_rules, ignore);

    ProtoOption {
      name: BUF_VALIDATE_FIELD.clone(),
      value: OptionValue::Message(outer_rules.into()),
    }
  }
}

reusable_string!(TIMESTAMP);
