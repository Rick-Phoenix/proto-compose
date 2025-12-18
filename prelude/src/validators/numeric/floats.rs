use std::{fmt::Display, marker::PhantomData};

use bon::Builder;
use float_validator_builder::{IsComplete, IsUnset, SetIgnore, State};
use protocheck_core::ordered_float::{self, FloatCore};

use super::*;

pub(crate) trait IsDefault: Default + PartialEq {
  fn is_default(&self) -> bool {
    (*self) == Self::default()
  }
}

pub(crate) fn format_list<T: Display, I: IntoIterator<Item = T>>(list: I) -> String {
  let mut string = String::new();
  let mut iter = list.into_iter().peekable();

  while let Some(item) = iter.next() {
    write!(string, "{item}").unwrap();

    if iter.peek().is_some() {
      string.push_str(", ");
    }
  }

  string
}

impl<T: Default + PartialEq> IsDefault for T {}

impl<Num> Validator<Num> for FloatValidator<Num>
where
  Num: FloatWrapper,
{
  type Target = Num::RustType;

  impl_rules_collection!();
  impl_cel_check!();

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
      if self.finite && !val.is_finite() {
        violations.add(
          field_context,
          parent_elements,
          Num::FINITE_VIOLATION,
          "must be a finite number",
        );
      }

      if let Some(const_val) = self.const_ && val != const_val {
        violations.add(field_context, parent_elements, Num::CONST_VIOLATION, &format!("must be equal to {const_val}"));
      }

      if let Some(gt) = self.gt && val <= gt {
        violations.add(field_context, parent_elements, Num::GT_VIOLATION, &format!("must be greater than {gt}"));
      }

      if let Some(gte) = self.gte && val < gte {
        violations.add(field_context, parent_elements, Num::GTE_VIOLATION, &format!("must be greater than or equal to {gte}"));
      }

      if let Some(lt) = self.lt && val >= lt {
        violations.add(field_context, parent_elements, Num::LT_VIOLATION, &format!("must be smaller than {lt}"));
      }

      if let Some(lte) = self.lte && val > lte {
        violations.add(field_context, parent_elements, Num::LTE_VIOLATION, &format!("must be smaller than or equal to {lte}"));
      }

      if let Some(allowed_list) = &self.in_ && !Num::RustType::is_in(allowed_list, val) {
        violations.add(field_context, parent_elements, Num::IN_VIOLATION, &format!("must be one of these values: {}", format_list(allowed_list.into_iter())));
      }

      if let Some(forbidden_list) = &self.not_in && Num::RustType::is_in(forbidden_list, val) {
        violations.add(field_context, parent_elements, Num::NOT_IN_VIOLATION, &format!("cannot be one of these values: {}", format_list(forbidden_list.into_iter())));
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
  pub in_: Option<ItemLookup<'static, OrderedFloat<Num::RustType>>>,
  /// Specifies that the values in this list will be considered NOT valid for this field.
  #[builder(into)]
  pub not_in: Option<ItemLookup<'static, OrderedFloat<Num::RustType>>>,
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  #[builder(default, with = |programs: impl IntoIterator<Item = &'static LazyLock<CelProgram>>| collect_programs(programs))]
  pub cel: Vec<&'static CelProgram>,
  /// Specifies that the field must be set in order to be valid.
  #[builder(default, with = || true)]
  pub required: bool,
  #[builder(setters(vis = "", name = ignore))]
  pub ignore: Option<Ignore>,
  /// Specifies that this field must be finite (i.e. it can't represent Infinity or NaN)
  #[builder(default, with = || true)]
  pub finite: bool,
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

    insert_boolean_option!(validator, values, finite);
    insert_option!(validator, values, lt);
    insert_option!(validator, values, lte);
    insert_option!(validator, values, gt);
    insert_option!(validator, values, gte);

    if let Some(allowed_list) = &validator.in_ {
      values.push((
        IN_.clone(),
        OptionValue::new_list(allowed_list.into_iter().map(|of| of.0)),
      ));
    }

    if let Some(forbidden_list) = &validator.not_in {
      values.push((
        NOT_IN.clone(),
        OptionValue::new_list(forbidden_list.into_iter().map(|of| of.0)),
      ));
    }

    let mut outer_rules: OptionValueList = vec![];

    outer_rules.push((N::type_name(), OptionValue::Message(values.into())));

    insert_cel_rules!(validator, outer_rules);
    insert_boolean_option!(validator, outer_rules, required);
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
  type RustType: PartialOrd
    + PartialEq
    + Copy
    + Into<OptionValue>
    + Debug
    + Display
    + Default
    + Into<::cel::Value>
    + ordered_float::FloatCore
    + ListRules<LookupTarget = OrderedFloat<Self::RustType>>
    + 'static;
  const LT_VIOLATION: &'static LazyLock<ViolationData>;
  const LTE_VIOLATION: &'static LazyLock<ViolationData>;
  const GT_VIOLATION: &'static LazyLock<ViolationData>;
  const GTE_VIOLATION: &'static LazyLock<ViolationData>;
  const IN_VIOLATION: &'static LazyLock<ViolationData> = Self::RustType::IN_VIOLATION;
  const NOT_IN_VIOLATION: &'static LazyLock<ViolationData> = Self::RustType::NOT_IN_VIOLATION;
  const CONST_VIOLATION: &'static LazyLock<ViolationData>;
  const FINITE_VIOLATION: &'static LazyLock<ViolationData>;

  fn type_name() -> Arc<str>;
}

macro_rules! impl_float_wrapper {
  ($target_type:ty, $proto_type:ident) => {
    paste::paste! {
      impl FloatWrapper for $target_type {
        type RustType = $target_type;
        const LT_VIOLATION: &'static LazyLock<ViolationData> = &[< $proto_type _LT_VIOLATION >];
        const LTE_VIOLATION: &'static LazyLock<ViolationData> = &[< $proto_type _LTE_VIOLATION >];
        const GT_VIOLATION: &'static LazyLock<ViolationData> = &[< $proto_type _GT_VIOLATION >];
        const GTE_VIOLATION: &'static LazyLock<ViolationData> = &[< $proto_type _GTE_VIOLATION >];
        const CONST_VIOLATION: &'static LazyLock<ViolationData> = &[< $proto_type _CONST_VIOLATION >];
        const FINITE_VIOLATION: &'static LazyLock<ViolationData> = &[< $proto_type _FINITE_VIOLATION >];

        fn type_name() -> Arc<str> {
          $proto_type.clone()
        }
      }

      impl ProtoValidator<$target_type> for $target_type {
        type Target = $target_type;
        type Validator = FloatValidator<$target_type>;
        type Builder = FloatValidatorBuilder<$target_type>;

        fn builder() -> Self::Builder {
          FloatValidator::builder()
        }
      }

      impl<S: State> ValidatorBuilderFor<$target_type> for FloatValidatorBuilder<$target_type, S> {
        type Target = $target_type;
        type Validator = FloatValidator<$target_type>;

        fn build_validator(self) -> Self::Validator {
          self.build()
        }
      }
    }
  };
}

impl_float_wrapper!(f32, FLOAT);
impl_float_wrapper!(f64, DOUBLE);
