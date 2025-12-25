pub mod builder;
pub use builder::TimestampValidatorBuilder;
use builder::state::State;

use proto_types::{Duration, Timestamp};

use super::*;

impl_validator!(TimestampValidator, Timestamp);

impl Validator<Timestamp> for TimestampValidator {
  type Target = Timestamp;
  type UniqueStore<'a>
    = CopyHybridStore<Timestamp>
  where
    Self: 'a;

  impl_testing_methods!();

  fn make_unique_store<'a>(&self, size: usize) -> Self::UniqueStore<'a> {
    CopyHybridStore::default_with_capacity(size)
  }

  #[cfg(feature = "testing")]
  fn check_consistency(&self) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if let Err(e) = check_comparable_rules(self.lt, self.lte, self.gt, self.gte) {
      errors.push(e.to_string());
    }

    if self.gt_now && (self.gt.is_some() || self.gte.is_some()) {
      errors.push("`gt_now` cannot be used with `gt` or `gte`".to_string());
    }

    if self.lt_now && (self.lt.is_some() || self.lte.is_some()) {
      errors.push("`lt_now` cannot be used with `lt` or `lte`".to_string());
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

    let mut violations_agg = Violations::new();
    let violations = &mut violations_agg;

    if let Some(&val) = val {
      if let Some(const_val) = self.const_
        && val != const_val
      {
        violations.add(
          field_context,
          parent_elements,
          &TIMESTAMP_CONST_VIOLATION,
          &format!("must be equal to {const_val}"),
        );
      }

      if self.gt_now && !val.is_future() {
        violations.add(
          field_context,
          parent_elements,
          &TIMESTAMP_GT_NOW_VIOLATION,
          "must be in the future",
        );
      }

      if self.lt_now && !val.is_past() {
        violations.add(
          field_context,
          parent_elements,
          &TIMESTAMP_LT_NOW_VIOLATION,
          "must be in the past",
        );
      }

      if let Some(range) = self.within
        && !val.is_within_range_from_now(range)
      {
        violations.add(
          field_context,
          parent_elements,
          &TIMESTAMP_WITHIN_VIOLATION,
          &format!("must be within {range} from now"),
        );
      }

      if let Some(gt) = self.gt
        && val <= gt
      {
        violations.add(
          field_context,
          parent_elements,
          &TIMESTAMP_GT_VIOLATION,
          &format!("must be later than {gt}"),
        );
      }

      if let Some(gte) = self.gte
        && val < gte
      {
        violations.add(
          field_context,
          parent_elements,
          &TIMESTAMP_GTE_VIOLATION,
          &format!("must be later than or equal to {gte}"),
        );
      }

      if let Some(lt) = self.lt
        && val >= lt
      {
        violations.add(
          field_context,
          parent_elements,
          &TIMESTAMP_LT_VIOLATION,
          &format!("must be earlier than {lt}"),
        );
      }

      if let Some(lte) = self.lte
        && val > lte
      {
        violations.add(
          field_context,
          parent_elements,
          &TIMESTAMP_LTE_VIOLATION,
          &format!("must be earlier than or equal to {lte}"),
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

#[derive(Clone, Debug)]
pub struct TimestampValidator {
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  pub cel: Vec<&'static CelProgram>,

  pub ignore: Option<Ignore>,

  /// Specifies that this field's value will be valid only if it in the past.
  pub lt_now: bool,

  /// Specifies that this field's value will be valid only if it in the future.
  pub gt_now: bool,

  /// Specifies that the field must be set in order to be valid.
  pub required: bool,

  /// Specifies that only this specific value will be considered valid for this field.
  pub const_: Option<Timestamp>,

  /// Specifies that this field's value will be valid only if it is smaller than the specified amount.
  pub lt: Option<Timestamp>,

  /// Specifies that this field's value will be valid only if it is smaller than, or equal to, the specified amount.
  pub lte: Option<Timestamp>,

  /// Specifies that this field's value will be valid only if it is greater than the specified amount.
  pub gt: Option<Timestamp>,

  /// Specifies that this field's value will be valid only if it is greater than, or equal to, the specified amount.
  pub gte: Option<Timestamp>,

  /// Specifies that this field's value will be valid only if it is within the specified Duration (either in the past or future) from the moment when it's being validated.
  pub within: Option<Duration>,
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
    insert_boolean_option!(validator, rules, lt_now);
    insert_boolean_option!(validator, rules, gt_now);
    insert_option!(validator, rules, within);

    let mut outer_rules: OptionValueList = vec![];

    outer_rules.push((TIMESTAMP.clone(), OptionValue::Message(rules.into())));

    insert_cel_rules!(validator, outer_rules);
    insert_boolean_option!(validator, outer_rules, required);
    insert_option!(validator, outer_rules, ignore);

    Self {
      name: BUF_VALIDATE_FIELD.clone(),
      value: OptionValue::Message(outer_rules.into()),
    }
  }
}
