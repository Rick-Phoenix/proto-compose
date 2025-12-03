use bon::Builder;
use enum_validator_builder::State;

use super::*;

pub struct GenericProtoEnum;

impl_validator!(EnumValidator, GenericProtoEnum);
impl_ignore!(EnumValidatorBuilder);
impl_into_option!(EnumValidator);

#[derive(Clone, Debug, Builder)]
#[builder(derive(Clone))]
pub struct EnumValidator {
  /// Specifies that only the values in this list will be considered valid for this field.
  #[builder(into)]
  pub in_: Option<Arc<[i32]>>,
  /// Specifies that the values in this list will be considered NOT valid for this field.
  #[builder(into)]
  pub not_in: Option<Arc<[i32]>>,
  /// Specifies that only this specific value will be considered valid for this field.
  pub const_: Option<i32>,
  #[builder(with = || true)]
  /// Marks that this field will only accept values that are defined in the enum that it's referring to.
  pub defined_only: Option<bool>,
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  pub cel: Option<Arc<[CelRule]>>,
  #[builder(with = || true)]
  /// Specifies that the field must be set in order to be valid.
  pub required: Option<bool>,
  #[builder(setters(vis = "", name = ignore))]
  pub ignore: Option<Ignore>,
}

impl From<EnumValidator> for ProtoOption {
  #[track_caller]
  fn from(validator: EnumValidator) -> Self {
    let mut rules: OptionValueList = Vec::new();

    if let Some(const_val) = validator.const_ {
      rules.push((CONST_.clone(), OptionValue::Int(const_val as i64)));
    }

    validate_lists(validator.in_.as_deref(), validator.not_in.as_deref()).unwrap();

    insert_option!(validator, rules, defined_only);
    insert_option!(validator, rules, in_);
    insert_option!(validator, rules, not_in);

    let mut outer_rules: OptionValueList = vec![];

    outer_rules.push((ENUM.clone(), OptionValue::Message(rules.into())));

    insert_cel_rules!(validator, outer_rules);
    insert_option!(validator, outer_rules, required);
    insert_option!(validator, outer_rules, ignore);

    ProtoOption {
      name: BUF_VALIDATE_FIELD.clone(),
      value: OptionValue::Message(outer_rules.into()),
    }
  }
}
