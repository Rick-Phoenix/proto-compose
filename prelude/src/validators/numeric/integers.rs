use std::{fmt::Display, marker::PhantomData};

use bon::Builder;
use int_validator_builder::{IsComplete, IsUnset, SetIgnore, State};
use proto_types::protovalidate::violations_data::*;
pub use protocheck_core::wrappers::{Fixed32, Fixed64, Sfixed32, Sfixed64, Sint32, Sint64};

use super::*;
use crate::field_context::ViolationsExt;

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

  impl_testing_methods!();

  #[cfg(feature = "testing")]
  fn check_consistency(&self) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if let Err(e) = self.check_cel_programs() {
      errors.extend(e.into_iter().map(|e| e.to_string()));
    }

    if let Err(e) = check_list_rules(self.in_, self.not_in) {
      errors.push(e);
    }

    if let Err(e) = check_comparable_rules(self.lt, self.lte, self.gt, self.gte) {
      errors.push(e.to_string());
    }

    if errors.is_empty() {
      Ok(())
    } else {
      Err(errors)
    }
  }

  fn validate(
    &self,
    field_context: &FieldContext,
    parent_elements: &mut Vec<FieldPathElement>,
    val: Option<&Self::Target>,
  ) -> Result<(), Violations> {
    handle_ignore_always!(&self.ignore);
    handle_ignore_if_zero_value!(&self.ignore, val.is_none_or(|v| v.is_default()));

    let mut violations_agg = Violations::new();
    let violations = &mut violations_agg;

    if let Some(&val) = val {
      if let Some(const_val) = self.const_
        && val != const_val
      {
        violations.add(
          field_context,
          parent_elements,
          Num::CONST_VIOLATION,
          &format!("must be equal to {const_val}"),
        );
      }

      if let Some(gt) = self.gt
        && val <= gt
      {
        violations.add(
          field_context,
          parent_elements,
          Num::GT_VIOLATION,
          &format!("must be greater than {gt}"),
        );
      }

      if let Some(gte) = self.gte
        && val < gte
      {
        violations.add(
          field_context,
          parent_elements,
          Num::GTE_VIOLATION,
          &format!("must be greater than or equal to {gte}"),
        );
      }

      if let Some(lt) = self.lt
        && val >= lt
      {
        violations.add(
          field_context,
          parent_elements,
          Num::LT_VIOLATION,
          &format!("must be smaller than {lt}"),
        );
      }

      if let Some(lte) = self.lte
        && val > lte
      {
        violations.add(
          field_context,
          parent_elements,
          Num::LTE_VIOLATION,
          &format!("must be smaller than or equal to {lte}"),
        );
      }

      if let Some(allowed_list) = &self.in_
        && !Num::RustType::is_in(allowed_list, val)
      {
        violations.add(
          field_context,
          parent_elements,
          Num::IN_VIOLATION,
          &format!(
            "must be one of these values: {}",
            format_list(allowed_list.iter())
          ),
        );
      }

      if let Some(forbidden_list) = &self.not_in
        && Num::RustType::is_in(forbidden_list, val)
      {
        violations.add(
          field_context,
          parent_elements,
          Num::NOT_IN_VIOLATION,
          &format!(
            "cannot be one of these values: {}",
            format_list(forbidden_list.iter())
          ),
        );
      }

      if !self.cel.is_empty() {
        let ctx = ProgramsExecutionCtx {
          programs: &self.cel,
          value: val,
          violations,
          field_context: Some(field_context),
          parent_elements,
        };

        ctx.execute_programs();
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

#[derive(Clone, Debug, Builder)]
pub struct IntValidator<Num>
where
  Num: IntWrapper,
{
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  #[builder(field)]
  pub cel: Vec<&'static CelProgram>,

  #[builder(setters(vis = "", name = ignore))]
  pub ignore: Option<Ignore>,

  #[builder(default)]
  _wrapper: PhantomData<Num>,

  /// Specifies that the field must be set in order to be valid.
  #[builder(default, with = || true)]
  pub required: bool,

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
  pub in_: Option<&'static ItemLookup<Num::RustType>>,

  /// Specifies that the values in this list will be considered NOT valid for this field.
  pub not_in: Option<&'static ItemLookup<Num::RustType>>,
}

impl<S: State, N: IntWrapper> IntValidatorBuilder<N, S> {
  /// Adds a custom CEL rule to this validator.
  /// Use the [`cel_program`] or [`inline_cel_program`] macros to build a static program.
  pub fn cel(mut self, program: &'static CelProgram) -> Self {
    self.cel.push(program);
    self
  }

  /// Rules defined for this field will be ignored if the field is set to its protobuf zero value.
  pub fn ignore_if_zero_value(self) -> IntValidatorBuilder<N, SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    self.ignore(Ignore::IfZeroValue)
  }

  /// Rules set for this field will always be ignored.
  pub fn ignore_always(self) -> IntValidatorBuilder<N, SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    self.ignore(Ignore::Always)
  }
}

impl<S, N> From<IntValidatorBuilder<N, S>> for ProtoOption
where
  S: State + IsComplete,
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

    if let Some(allowed_list) = &validator.in_ {
      values.push((IN_.clone(), OptionValue::new_list(allowed_list.iter())));
    }

    if let Some(forbidden_list) = &validator.not_in {
      values.push((NOT_IN.clone(), OptionValue::new_list(forbidden_list.iter())));
    }

    let mut outer_rules: OptionValueList =
      vec![(N::type_name(), OptionValue::Message(values.into()))];

    insert_cel_rules!(validator, outer_rules);
    insert_boolean_option!(validator, outer_rules, required);
    insert_option!(validator, outer_rules, ignore);

    Self {
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
    + ListRules<LookupTarget = Self::RustType>
    + Into<::cel::Value>
    + 'static;
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
      impl_int_wrapper!($rust_type, $rust_type, $proto_type);
      impl_proto_type!($rust_type, stringify!([ < $proto_type:lower > ]));
      impl_int_validator!($rust_type, $rust_type);
    }
  };

  ($rust_type:ty, $wrapper:ident) => {
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
      impl ProtoValidator for $wrapper {
        type Target = $rust_type;
        type Validator = IntValidator<$wrapper>;
        type Builder = IntValidatorBuilder<$wrapper>;

        fn validator_builder() -> IntValidatorBuilder<$wrapper> {
          IntValidator::builder()
        }
      }
    }
  };
}

impl_int!(i32, Sint32);
impl_int!(i64, Sint64);
impl_int!(i32, Sfixed32);
impl_int!(i64, Sfixed64);
impl_int!(u32, Fixed32);
impl_int!(u64, Fixed64);
impl_int!(i32, INT32, primitive);
impl_int!(i64, INT64, primitive);
impl_int!(u32, UINT32, primitive);
impl_int!(u64, UINT64, primitive);
