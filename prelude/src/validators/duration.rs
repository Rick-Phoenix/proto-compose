use bon::Builder;
use duration_validator_builder::{IsUnset, SetIgnore, State};
use proto_types::Duration;

use super::*;

impl_validator!(DurationValidator, Duration);
impl_into_option!(DurationValidator);

impl Validator<Duration> for DurationValidator {
  type Target = Duration;
}

#[derive(Clone, Debug, Builder)]
#[builder(derive(Clone))]
pub struct DurationValidator {
  /// Specifies that only the values in this list will be considered valid for this field.
  #[builder(into)]
  pub in_: Option<Arc<[Duration]>>,
  /// Specifies that the values in this list will be considered NOT valid for this field.
  #[builder(into)]
  pub not_in: Option<Arc<[Duration]>>,
  /// Specifies that only this specific value will be considered valid for this field.
  pub const_: Option<Duration>,
  /// Specifies that the value must be smaller than the indicated amount in order to pass validation.
  pub lt: Option<Duration>,
  /// Specifies that the value must be equal to or smaller than the indicated amount in order to pass validation.
  pub lte: Option<Duration>,
  /// Specifies that the value must be greater than the indicated amount in order to pass validation.
  pub gt: Option<Duration>,
  /// Specifies that the value must be equal to or greater than the indicated amount in order to pass validation.
  pub gte: Option<Duration>,
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  #[builder(into)]
  pub cel: Option<Arc<[CelRule]>>,
  #[builder(default, with = || true)]
  /// Specifies that the field must be set in order to be valid.
  pub required: bool,
  #[builder(setters(vis = "", name = ignore))]
  pub ignore: Option<Ignore>,
}

impl<S: State> DurationValidatorBuilder<S>
where
  S::Ignore: IsUnset,
{
  /// Rules set for this field will always be ignored.
  pub fn ignore_always(self) -> DurationValidatorBuilder<SetIgnore<S>> {
    self.ignore(Ignore::Always)
  }
}

impl From<DurationValidator> for ProtoOption {
  fn from(validator: DurationValidator) -> Self {
    let mut rules: OptionValueList = Vec::new();

    if let Some(const_val) = validator.const_ {
      rules.push((CONST_.clone(), OptionValue::Duration(const_val)));
    }

    insert_option!(validator, rules, lt);
    insert_option!(validator, rules, lte);
    insert_option!(validator, rules, gt);
    insert_option!(validator, rules, gte);
    insert_option!(validator, rules, in_);
    insert_option!(validator, rules, not_in);

    let mut outer_rules: OptionValueList = vec![];

    outer_rules.push((DURATION.clone(), OptionValue::Message(rules.into())));

    insert_cel_rules!(validator, outer_rules);
    insert_boolean_option!(validator, outer_rules, required);
    insert_option!(validator, outer_rules, ignore);

    ProtoOption {
      name: BUF_VALIDATE_FIELD.clone(),
      value: OptionValue::Message(outer_rules.into()),
    }
  }
}
