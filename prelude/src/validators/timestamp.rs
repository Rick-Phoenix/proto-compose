mod builder;
pub use builder::TimestampValidatorBuilder;

use proto_types::{Duration, Timestamp};

use super::*;

#[derive(Clone, Debug)]
pub struct TimestampValidator {
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  pub cel: Vec<CelProgram>,

  pub ignore: Ignore,

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

  pub now_tolerance: Duration,
}

impl Validator<Timestamp> for TimestampValidator {
  type Target = Timestamp;
  type UniqueStore<'a>
    = CopyHybridStore<Timestamp>
  where
    Self: 'a;

  impl_testing_methods!();

  #[inline]
  fn make_unique_store<'a>(&self, size: usize) -> Self::UniqueStore<'a> {
    CopyHybridStore::default_with_capacity(size)
  }

  fn check_consistency(&self) -> Result<(), Vec<ConsistencyError>> {
    let mut errors = Vec::new();

    macro_rules! check_prop_some {
      ($($id:ident),*) => {
        $(self.$id.is_some()) ||*
      };
    }

    if self.const_.is_some()
      && (!self.cel.is_empty()
        || self.lt_now
        || self.gt_now
        || check_prop_some!(within, lt, lte, gt, gte))
    {
      errors.push(ConsistencyError::ConstWithOtherRules);
    }

    #[cfg(feature = "cel")]
    if let Err(e) = self.check_cel_programs() {
      errors.extend(e.into_iter().map(ConsistencyError::from));
    }

    if let Err(e) = check_comparable_rules(self.lt, self.lte, self.gt, self.gte) {
      errors.push(e);
    }

    if self.gt_now && self.lt_now {
      errors.push(ConsistencyError::ContradictoryInput(
        "`lt_now` and `gt_now` cannot be used together".to_string(),
      ));
    }

    if self.gt_now && (self.gt.is_some() || self.gte.is_some()) {
      errors.push(ConsistencyError::ContradictoryInput(
        "`gt_now` cannot be used with `gt` or `gte`".to_string(),
      ));
    }

    if self.lt_now && (self.lt.is_some() || self.lte.is_some()) {
      errors.push(ConsistencyError::ContradictoryInput(
        "`lt_now` cannot be used with `lt` or `lte`".to_string(),
      ));
    }

    if errors.is_empty() {
      Ok(())
    } else {
      Err(errors)
    }
  }

  fn validate(&self, ctx: &mut ValidationCtx, val: Option<&Self::Target>) {
    handle_ignore_always!(&self.ignore);

    if let Some(&val) = val {
      if let Some(const_val) = self.const_ {
        if val != const_val {
          ctx.add_violation(
            &TIMESTAMP_CONST_VIOLATION,
            &format!("must be equal to {const_val}"),
          );
        }

        // Using `const` implies no other rules
        return;
      }

      if self.gt_now && !(val + self.now_tolerance).is_future() {
        ctx.add_violation(&TIMESTAMP_GT_NOW_VIOLATION, "must be in the future");
      }

      if self.lt_now && !val.is_past() {
        ctx.add_violation(&TIMESTAMP_LT_NOW_VIOLATION, "must be in the past");
      }

      if let Some(range) = self.within
        && !val.is_within_range_from_now(range)
      {
        ctx.add_violation(
          &TIMESTAMP_WITHIN_VIOLATION,
          &format!("must be within {range} from now"),
        );
      }

      if let Some(gt) = self.gt
        && val <= gt
      {
        ctx.add_violation(&TIMESTAMP_GT_VIOLATION, &format!("must be later than {gt}"));
      }

      if let Some(gte) = self.gte
        && val < gte
      {
        ctx.add_violation(
          &TIMESTAMP_GTE_VIOLATION,
          &format!("must be later than or equal to {gte}"),
        );
      }

      if let Some(lt) = self.lt
        && val >= lt
      {
        ctx.add_violation(
          &TIMESTAMP_LT_VIOLATION,
          &format!("must be earlier than {lt}"),
        );
      }

      if let Some(lte) = self.lte
        && val > lte
      {
        ctx.add_violation(
          &TIMESTAMP_LTE_VIOLATION,
          &format!("must be earlier than or equal to {lte}"),
        );
      }

      #[cfg(feature = "cel")]
      if !self.cel.is_empty() {
        let ctx = ProgramsExecutionCtx {
          programs: &self.cel,
          value: val,
          violations: ctx.violations,
          field_context: Some(&ctx.field_context),
          parent_elements: ctx.parent_elements,
        };

        ctx.execute_programs();
      }
    } else if self.required {
      ctx.add_required_violation();
    }
  }
}

impl From<TimestampValidator> for ProtoOption {
  fn from(validator: TimestampValidator) -> Self {
    let mut rules = OptionMessageBuilder::new();

    macro_rules! set_options {
      ($($name:ident),*) => {
        rules
        $(
          .maybe_set(stringify!($name), validator.$name)
        )*
      };
    }

    set_options!(lt, lte, gt, gte, within);

    rules
      .maybe_set("const", validator.const_)
      .set_boolean("lt_now", validator.lt_now)
      .set_boolean("gt_now", validator.gt_now);

    let mut outer_rules = OptionMessageBuilder::new();

    outer_rules.set("timestamp", OptionValue::Message(rules.into()));

    outer_rules
      .add_cel_options(validator.cel)
      .set_required(validator.required)
      .set_ignore(validator.ignore);

    Self {
      name: "(buf.validate.field)".into(),
      value: OptionValue::Message(outer_rules.into()),
    }
  }
}
