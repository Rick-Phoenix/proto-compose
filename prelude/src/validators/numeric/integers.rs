use std::{fmt::Display, marker::PhantomData};

use bon::Builder;
use int_validator_builder::{IsComplete, IsUnset, SetIgnore, State};

use super::*;

impl<Num> Validator for IntValidator<Num>
where
  Num: IntWrapper,
{
  type Target = Num::RustType;

  fn validate(&self, _val: &Self::Target) -> Result<(), bool> {
    Ok(())
  }
}

impl<Num, S: State> Validator for IntValidatorBuilder<Num, S>
where
  Num: IntWrapper,
{
  type Target = Num::RustType;

  fn validate(&self, _val: &Self::Target) -> Result<(), bool> {
    Ok(())
  }
}

#[derive(Clone, Debug, Builder)]
pub struct IntValidator<Num>
where
  Num: IntWrapper,
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
}

impl<S: State, N: IntWrapper> IntValidatorBuilder<N, S>
where
  S::Ignore: IsUnset,
{
  /// Rules defined for this field will be ignored if the field is set to its protobuf zero value.
  pub fn ignore_if_zero_value(self) -> IntValidatorBuilder<N, SetIgnore<S>> {
    self.ignore(Ignore::IfZeroValue)
  }

  /// Rules set for this field will always be ignored.
  pub fn ignore_always(self) -> IntValidatorBuilder<N, SetIgnore<S>> {
    self.ignore(Ignore::Always)
  }
}

impl<S: State, N> From<IntValidatorBuilder<N, S>> for ProtoOption
where
  S: IsComplete,
  N: IntWrapper,
{
  fn from(value: IntValidatorBuilder<N, S>) -> Self {
    value.build().into()
  }
}

impl<N> From<IntValidator<N>> for ProtoOption
where
  N: IntWrapper,
{
  fn from(validator: IntValidator<N>) -> Self {
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

    let mut outer_rules: OptionValueList =
      vec![(N::type_name(), OptionValue::Message(values.into()))];

    insert_cel_rules!(validator, outer_rules);
    insert_option!(validator, outer_rules, required);
    insert_option!(validator, outer_rules, ignore);

    ProtoOption {
      name: BUF_VALIDATE_FIELD.clone(),
      value: OptionValue::Message(outer_rules.into()),
    }
  }
}

pub trait IntWrapper: AsProtoType {
  type RustType: PartialOrd + PartialEq + Copy + Into<OptionValue> + Hash + Debug + Display + Eq;

  fn type_name() -> Arc<str>;
}

macro_rules! impl_int_wrapper {
  ($rust_type:ty, $proto_type:ident, primitive) => {
    impl IntWrapper for $rust_type {
      type RustType = $rust_type;

      fn type_name() -> Arc<str> {
        $crate::paste!([< $proto_type:upper >]).clone()
      }
    }

    impl_proto_type!($rust_type, stringify!($proto_type));
    impl_int_validator!($rust_type);
  };

  ($rust_type:ty, $proto_type:ident) => {
    pub struct $proto_type;

    impl IntWrapper for $proto_type {
      type RustType = $rust_type;

      fn type_name() -> Arc<str> {
        $crate::paste!([< $proto_type:upper >]).clone()
      }
    }

    $crate::paste!(
      impl_proto_type!($proto_type, stringify!([< $proto_type:lower >]));
      impl_int_validator!($proto_type);
    );
  };
}

macro_rules! impl_int_validator {
  ($rust_type:ty) => {
    $crate::paste! {
      impl ProtoValidator<$rust_type> for $rust_type {
        type Validator = IntValidator<$rust_type>;
        type Builder = IntValidatorBuilder<$rust_type>;

        fn builder() -> IntValidatorBuilder<$rust_type> {
          IntValidator::builder()
        }
      }

      impl<S: State> ValidatorBuilderFor<$rust_type>
      for IntValidatorBuilder<$rust_type, S>
      {
        type Validator = IntValidator<$rust_type>;

        fn build_validator(self) -> Self::Validator {
          self.build()
        }
      }
    }
  };
}

impl_int_wrapper!(i32, Sint32);
impl_int_wrapper!(i32, INT32, primitive);
