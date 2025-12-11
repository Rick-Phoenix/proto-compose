use bon::Builder;
use enum_validator_builder::{IsUnset, SetIgnore, State};

use super::*;

impl<T: ProtoEnum, S: State> ValidatorBuilderFor<T> for EnumValidatorBuilder<T, S> {
  type Target = i32;
  type Validator = EnumValidator<T>;

  fn build_validator(self) -> Self::Validator {
    self.build()
  }
}

impl<T: ProtoEnum> Validator<T> for EnumValidator<T> {
  type Target = i32;

  // fn validate(&self, val: Option<&i32>) -> Result<(), bool> {
  //   if let Some(val) = val {
  //     if self.defined_only && T::try_from(*val).is_err() {
  //       println!("Must be a defined value");
  //     }
  //   }
  //
  //   Ok(())
  // }
}

#[derive(Clone, Debug, Builder)]
#[builder(derive(Clone))]
pub struct EnumValidator<T: ProtoEnum> {
  #[builder(default, setters(vis = ""))]
  _enum: PhantomData<T>,
  /// Specifies that only the values in this list will be considered valid for this field.
  #[builder(into)]
  pub in_: Option<Arc<[i32]>>,
  /// Specifies that the values in this list will be considered NOT valid for this field.
  #[builder(into)]
  pub not_in: Option<Arc<[i32]>>,
  /// Specifies that only this specific value will be considered valid for this field.
  pub const_: Option<i32>,
  #[builder(default, with = || true)]
  /// Marks that this field will only accept values that are defined in the enum that it's referring to.
  pub defined_only: bool,
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  pub cel: Option<Arc<[CelRule]>>,
  #[builder(default, with = || true)]
  /// Specifies that the field must be set in order to be valid.
  pub required: bool,
  #[builder(setters(vis = "", name = ignore))]
  pub ignore: Option<Ignore>,
}

impl<T: ProtoEnum, S: State> EnumValidatorBuilder<T, S> {
  #[doc = r" Rules defined for this field will be ignored if the field is set to its protobuf zero value."]
  pub fn ignore_if_zero_value(self) -> EnumValidatorBuilder<T, SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    self.ignore(Ignore::IfZeroValue)
  }

  #[doc = r" Rules set for this field will always be ignored."]
  pub fn ignore_always(self) -> EnumValidatorBuilder<T, SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    self.ignore(Ignore::Always)
  }
}

impl<T: ProtoEnum, S: State> From<EnumValidatorBuilder<T, S>> for ProtoOption {
  fn from(value: EnumValidatorBuilder<T, S>) -> Self {
    value.build().into()
  }
}

impl<T: ProtoEnum> From<EnumValidator<T>> for ProtoOption {
  fn from(validator: EnumValidator<T>) -> Self {
    let mut rules: OptionValueList = Vec::new();

    if let Some(const_val) = validator.const_ {
      rules.push((CONST_.clone(), OptionValue::Int(const_val as i64)));
    }

    insert_boolean_option!(validator, rules, defined_only);
    insert_option!(validator, rules, in_);
    insert_option!(validator, rules, not_in);

    let mut outer_rules: OptionValueList = vec![];

    outer_rules.push((ENUM.clone(), OptionValue::Message(rules.into())));

    insert_cel_rules!(validator, outer_rules);
    insert_boolean_option!(validator, outer_rules, required);
    insert_option!(validator, outer_rules, ignore);

    ProtoOption {
      name: BUF_VALIDATE_FIELD.clone(),
      value: OptionValue::Message(outer_rules.into()),
    }
  }
}
