use std::{fmt::Display, marker::PhantomData};

use bon::Builder;
use int_validator_builder::{IsComplete, IsUnset, SetIgnore, State};
use proto_types::protovalidate::violations_data::*;

use super::*;
use crate::field_context::Violations;

impl<S: State, Num: IntWrapper> ValidatorBuilderFor<Num> for IntValidatorBuilder<Num, S> {
  type Target = Num::RustType;
  type Validator = IntValidator<Num>;

  fn build_validator(self) -> Self::Validator {
    self.build()
  }
}

impl<Num> Validator<Num> for IntValidator<Num>
where
  Num: IntWrapper,
{
  type Target = Num::RustType;

  fn cel_rules(&self) -> Vec<&'static CelRule> {
    self.cel.iter().map(|prog| &prog.rule).collect()
  }

  fn validate_cel_with(&self, val: Self::Target) -> Result<(), Vec<CelError>> {
    if !self.cel.is_empty() {
      test_programs(&self.cel, val)
    } else {
      Ok(())
    }
  }

  fn validate_cel(&self) -> Result<(), Vec<CelError>> {
    let val = Self::Target::default();

    self.validate_cel_with(val)
  }

  fn validate(
    &self,
    field_context: &FieldContext,
    parent_elements: &mut Vec<FieldPathElement>,
    val: Option<&Self::Target>,
  ) -> Result<(), Vec<Violation>> {
    let mut violations_agg: Vec<Violation> = Vec::new();
    let violations = &mut violations_agg;

    if let Some(&val) = val {
      if let Some(gt) = self.gt && val <= gt {
        violations.add(field_context, parent_elements, Num::GT_VIOLATION, &format!("must be greater than {gt}"));
      }

      if !self.cel.is_empty() {
        execute_cel_programs(ProgramsExecutionCtx {
          programs: &self.cel,
          value: val,
          violations,
          field_context: Some(field_context),
          parent_elements,
        });
      }
    } else if self.required {
      violations.add_required(field_context, parent_elements);
    }

    if violations.is_empty() {
      Ok(())
    } else {
      Err(violations_agg)
    }
  }
}

impl<Num, S: State> Validator<Num> for IntValidatorBuilder<Num, S>
where
  Num: IntWrapper,
{
  type Target = Num::RustType;
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
  #[builder(default, with = |programs: impl IntoIterator<Item = &'static LazyLock<CelProgram>>| collect_programs(programs))]
  pub cel: Vec<&'static CelProgram>,
  /// Specifies that the field must be set in order to be valid.
  #[builder(default, with = || true)]
  pub required: bool,
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
    insert_boolean_option!(validator, outer_rules, required);
    insert_option!(validator, outer_rules, ignore);

    ProtoOption {
      name: BUF_VALIDATE_FIELD.clone(),
      value: OptionValue::Message(outer_rules.into()),
    }
  }
}

pub trait IntWrapper: AsProtoType {
  type RustType: PartialOrd
    + PartialEq
    + Copy
    + Into<OptionValue>
    + Hash
    + Debug
    + Display
    + Eq
    + Default
    + Into<::cel::Value>;
  const LT_VIOLATION: &'static LazyLock<ViolationData>;
  const LTE_VIOLATION: &'static LazyLock<ViolationData>;
  const GT_VIOLATION: &'static LazyLock<ViolationData>;
  const GTE_VIOLATION: &'static LazyLock<ViolationData>;
  const IN_VIOLATION: &'static LazyLock<ViolationData>;
  const NOT_IN_VIOLATION: &'static LazyLock<ViolationData>;
  const CONST_VIOLATION: &'static LazyLock<ViolationData>;

  fn type_name() -> Arc<str>;
}

macro_rules! impl_int_wrapper {
  ($wrapper:ty, $target_type:ty, $proto_type:ident) => {
    paste::paste! {
      impl IntWrapper for $wrapper {
        type RustType = $target_type;
        const LT_VIOLATION: &'static LazyLock<ViolationData> = &[< $proto_type _LT_VIOLATION >];
        const LTE_VIOLATION: &'static LazyLock<ViolationData> = &[< $proto_type _LTE_VIOLATION >];
        const GT_VIOLATION: &'static LazyLock<ViolationData> = &[< $proto_type _GT_VIOLATION >];
        const GTE_VIOLATION: &'static LazyLock<ViolationData> = &[< $proto_type _GTE_VIOLATION >];
        const IN_VIOLATION: &'static LazyLock<ViolationData> = &[< $proto_type _IN_VIOLATION >];
        const NOT_IN_VIOLATION: &'static LazyLock<ViolationData> = &[< $proto_type _NOT_IN_VIOLATION >];
        const CONST_VIOLATION: &'static LazyLock<ViolationData> = &[< $proto_type _CONST_VIOLATION >];

        fn type_name() -> Arc<str> {
          $proto_type.clone()
        }
      }
    }
  };
}

macro_rules! impl_int {
  ($rust_type:ty, $proto_type:ident, primitive) => {
    paste::paste! {
      impl_int_wrapper!($rust_type, $rust_type, [< $proto_type:upper >]);
      impl_proto_type!($rust_type, stringify!($proto_type));
      impl_int_validator!($rust_type, $rust_type);
    }
  };

  ($rust_type:ty, $wrapper:ident) => {
    pub struct $wrapper;

    paste::paste! {
      impl_int_wrapper!($wrapper, $rust_type, [< $wrapper:upper >]);
      impl_proto_type!($wrapper, stringify!([< $wrapper:lower >]));
      impl_int_validator!($wrapper, $rust_type);
    }
  };
}

macro_rules! impl_int_validator {
  ($wrapper:ty, $rust_type:ty) => {
    $crate::paste! {
      impl ProtoValidator<$wrapper> for $wrapper {
        type Target = $rust_type;
        type Validator = IntValidator<$wrapper>;
        type Builder = IntValidatorBuilder<$wrapper>;

        fn builder() -> IntValidatorBuilder<$wrapper> {
          IntValidator::builder()
        }
      }
    }
  };
}

impl_int!(i32, Sint32);
impl_int!(i32, INT32, primitive);
impl_int!(u32, UINT32, primitive);
