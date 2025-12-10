use std::{fmt::Display, marker::PhantomData};

use bon::Builder;
use float_validator_builder::{IsComplete, IsUnset, SetIgnore, State};

use super::*;

#[derive(Clone, Debug, Builder)]
pub struct FloatValidator<Num>
where
  Num: FloatWrapper,
{
  #[builder(default)]
  _wrapper: PhantomData<Num>,
  /// Specifies that only this specific value will be considered valid for this field.
  pub const_: Option<Num::RustType>,
  /// Specifies that this field's value will be valid only if it is smaller than the specified amount
  pub lt: Option<Num::RustType>,
  /// Specifies that this field's value will be valid only if it is smaller than, or equal to, the specified amount
  pub lte: Option<Num::RustType>,
  /// Specifies that this field's value will be valid only if it is greater than the specified amount
  pub gt: Option<Num::RustType>,
  /// Specifies that this field's value will be valid only if it is smaller than, or equal to, the specified amount
  pub gte: Option<Num::RustType>,
  /// Specifies that only the values in this list will be considered valid for this field.
  #[builder(into)]
  pub in_: Option<Arc<[Num::RustType]>>,
  /// Specifies that the values in this list will be considered NOT valid for this field.
  #[builder(into)]
  pub not_in: Option<Arc<[Num::RustType]>>,
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  #[builder(into)]
  pub cel: Option<Arc<[CelRule]>>,
  /// Specifies that the field must be set in order to be valid.
  pub required: Option<bool>,
  #[builder(setters(vis = "", name = ignore))]
  pub ignore: Option<Ignore>,
  /// Specifies that this field must be finite (i.e. it can't represent Infinity or NaN)
  #[builder(with = || true)]
  pub finite: Option<bool>,
}

impl<S: State, N: FloatWrapper> FloatValidatorBuilder<N, S>
where
  S::Ignore: IsUnset,
{
  /// Rules defined for this field will be ignored if the field is set to its protobuf zero value.
  pub fn ignore_if_zero_value(self) -> FloatValidatorBuilder<N, SetIgnore<S>> {
    self.ignore(Ignore::IfZeroValue)
  }

  /// Rules set for this field will always be ignored.
  pub fn ignore_always(self) -> FloatValidatorBuilder<N, SetIgnore<S>> {
    self.ignore(Ignore::Always)
  }
}

impl<S: State, N> From<FloatValidatorBuilder<N, S>> for ProtoOption
where
  S: IsComplete,
  N: FloatWrapper,
{
  fn from(value: FloatValidatorBuilder<N, S>) -> Self {
    value.build().into()
  }
}

impl<N> From<FloatValidator<N>> for ProtoOption
where
  N: FloatWrapper,
{
  fn from(validator: FloatValidator<N>) -> Self {
    let mut values: OptionValueList = Vec::new();

    if let Some(const_val) = validator.const_ {
      values.push((CONST_.clone(), const_val.into()));
    }

    insert_option!(validator, values, lt);
    insert_option!(validator, values, lte);
    insert_option!(validator, values, gt);
    insert_option!(validator, values, gte);
    insert_option!(validator, values, in_);
    insert_option!(validator, values, not_in);

    let mut outer_rules: OptionValueList = vec![];

    outer_rules.push((N::type_name(), OptionValue::Message(values.into())));

    insert_cel_rules!(validator, outer_rules);
    insert_option!(validator, outer_rules, required);
    insert_option!(validator, outer_rules, ignore);

    ProtoOption {
      name: BUF_VALIDATE_FIELD.clone(),
      value: OptionValue::Message(outer_rules.into()),
    }
  }
}

impl_proto_type!(f32, "float");
impl_proto_type!(f64, "double");

pub trait FloatWrapper: AsProtoType {
  type RustType: PartialOrd + PartialEq + Copy + Into<OptionValue> + Debug + Display;

  fn type_name() -> Arc<str>;
}

impl FloatWrapper for f32 {
  type RustType = f32;

  fn type_name() -> Arc<str> {
    FLOAT.clone()
  }
}

impl FloatWrapper for f64 {
  type RustType = f64;

  fn type_name() -> Arc<str> {
    DOUBLE.clone()
  }
}

impl<S: State> ValidatorBuilderFor<f32> for FloatValidatorBuilder<f32, S> {
  type Validator = FloatValidator<f32>;

  fn build_validator(self) -> Self::Validator {
    self.build()
  }
}

impl<S: State> ValidatorBuilderFor<f64> for FloatValidatorBuilder<f64, S> {
  type Validator = FloatValidator<f64>;

  fn build_validator(self) -> Self::Validator {
    self.build()
  }
}

impl Validator for FloatValidator<f32> {
  type Target = f32;

  fn validate(&self, val: &Self::Target) -> Result<(), bool> {
    Ok(())
  }
}

impl Validator for FloatValidator<f64> {
  type Target = f64;

  fn validate(&self, val: &Self::Target) -> Result<(), bool> {
    Ok(())
  }
}

impl ProtoValidator<f32> for f32 {
  type Validator = FloatValidator<f32>;
  type Builder = FloatValidatorBuilder<f32>;

  fn builder() -> Self::Builder {
    FloatValidator::builder()
  }
}

impl ProtoValidator<f64> for f64 {
  type Validator = FloatValidator<f64>;
  type Builder = FloatValidatorBuilder<f64>;

  fn builder() -> Self::Builder {
    FloatValidator::builder()
  }
}
