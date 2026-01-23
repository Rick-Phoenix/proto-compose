mod builder;
pub use builder::TimestampValidatorBuilder;

use proto_types::{Duration, Timestamp};

use super::*;

#[non_exhaustive]
#[derive(Clone, Debug, Default)]
pub struct TimestampValidator {
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  pub cel: Vec<CelProgram>,

  pub ignore: Ignore,

  #[cfg(all(feature = "chrono", any(feature = "std", feature = "chrono-wasm")))]
  /// Specifies that this field's value will be valid only if it in the past.
  pub lt_now: bool,

  #[cfg(all(feature = "chrono", any(feature = "std", feature = "chrono-wasm")))]
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

  #[cfg(all(feature = "chrono", any(feature = "std", feature = "chrono-wasm")))]
  /// Specifies that this field's value will be valid only if it is within the specified Duration (either in the past or future) from the moment when it's being validated.
  pub within: Option<Duration>,

  #[cfg(all(feature = "chrono", any(feature = "std", feature = "chrono-wasm")))]
  pub now_tolerance: Duration,

  pub error_messages: Option<ErrorMessages<TimestampViolation>>,
}

impl TimestampValidator {
  const fn has_props(&self) -> bool {
    let mut has_props =
      self.lt.is_some() || self.lte.is_some() || self.gt.is_some() || self.gte.is_some();

    #[cfg(all(feature = "chrono", any(feature = "std", feature = "chrono-wasm")))]
    {
      has_props =
        has_props || !self.cel.is_empty() || self.lt_now || self.gt_now || self.within.is_some();
    }

    #[cfg(not(feature = "chrono"))]
    {
      has_props = has_props || !self.cel.is_empty();
    }

    has_props
  }
}

impl Validator<Timestamp> for TimestampValidator {
  type Target = Timestamp;

  impl_testing_methods!();

  fn check_consistency(&self) -> Result<(), Vec<ConsistencyError>> {
    let mut errors = Vec::new();

    if self.const_.is_some() && self.has_props() {
      errors.push(ConsistencyError::ConstWithOtherRules);
    }

    if let Some(custom_messages) = self.error_messages.as_deref() {
      let mut unused_messages: Vec<String> = Vec::new();

      for key in custom_messages.keys() {
        macro_rules! check_unused_messages {
          ($($name:ident),*) => {
            paste! {
              match key {
                TimestampViolation::Required => self.required,
                TimestampViolation::Const => self.const_.is_some(),
                #[cfg(all(feature = "chrono", any(feature = "std", feature = "chrono-wasm")))]
                TimestampViolation::LtNow => self.lt_now,
                #[cfg(all(feature = "chrono", any(feature = "std", feature = "chrono-wasm")))]
                TimestampViolation::GtNow => self.gt_now,
                #[cfg(all(feature = "chrono", any(feature = "std", feature = "chrono-wasm")))]
                TimestampViolation::Within => self.within.is_some(),
                $(TimestampViolation::[< $name:camel >] => self.$name.is_some(),)*
                _ => true,
              }
            }
          };
        }

        let is_used = check_unused_messages!(lt, lte, gt, gte);

        if !is_used {
          unused_messages.push(format!("{key:?}"));
        }
      }

      if !unused_messages.is_empty() {
        errors.push(ConsistencyError::UnusedCustomMessages(unused_messages));
      }
    }

    #[cfg(feature = "cel")]
    if let Err(e) = self.check_cel_programs() {
      errors.extend(e.into_iter().map(ConsistencyError::from));
    }

    if let Err(e) = check_comparable_rules(self.lt, self.lte, self.gt, self.gte) {
      errors.push(e);
    }

    #[cfg(all(feature = "chrono", any(feature = "std", feature = "chrono-wasm")))]
    if self.gt_now && self.lt_now {
      errors.push(ConsistencyError::ContradictoryInput(
        "`lt_now` and `gt_now` cannot be used together".to_string(),
      ));
    }

    #[cfg(all(feature = "chrono", any(feature = "std", feature = "chrono-wasm")))]
    if self.gt_now && (self.gt.is_some() || self.gte.is_some()) {
      errors.push(ConsistencyError::ContradictoryInput(
        "`gt_now` cannot be used with `gt` or `gte`".to_string(),
      ));
    }

    #[cfg(all(feature = "chrono", any(feature = "std", feature = "chrono-wasm")))]
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

  fn validate_core<V>(&self, ctx: &mut ValidationCtx, val: Option<&V>) -> ValidatorResult
  where
    V: Borrow<Self::Target> + ?Sized,
  {
    handle_ignore_always!(&self.ignore);

    let mut is_valid = IsValid::Yes;

    macro_rules! handle_violation {
      ($id:ident, $default:expr) => {
        is_valid &= ctx.add_timestamp_violation(
          TimestampViolation::$id,
          self
            .error_messages
            .as_deref()
            .and_then(|map| map.get(&TimestampViolation::$id))
            .map(|m| Cow::Borrowed(m.as_ref()))
            .unwrap_or_else(|| Cow::Owned($default)),
        )?;
      };
    }

    if let Some(val) = val {
      let val = *val.borrow();

      if let Some(const_val) = self.const_ {
        if val != const_val {
          handle_violation!(Const, format!("must be equal to {const_val}"));
        }

        // Using `const` implies no other rules
        return Ok(is_valid);
      }

      if let Some(gt) = self.gt
        && val <= gt
      {
        handle_violation!(Gt, format!("must be later than {gt}"));
      }

      if let Some(gte) = self.gte
        && val < gte
      {
        handle_violation!(Gte, format!("must be later than or equal to {gte}"));
      }

      if let Some(lt) = self.lt
        && val >= lt
      {
        handle_violation!(Lt, format!("must be earlier than {lt}"));
      }

      if let Some(lte) = self.lte
        && val > lte
      {
        handle_violation!(Lte, format!("must be earlier than or equal to {lte}"));
      }

      #[cfg(all(feature = "chrono", any(feature = "std", feature = "chrono-wasm")))]
      {
        if self.gt_now && !(val + self.now_tolerance).is_future() {
          handle_violation!(GtNow, "must be in the future".to_string());
        }

        if self.lt_now && !val.is_past() {
          handle_violation!(LtNow, "must be in the past".to_string());
        }

        if let Some(range) = self.within
          && !val.is_within_range_from_now(range)
        {
          handle_violation!(Within, format!("must be within {range} from now"));
        }
      }

      #[cfg(feature = "cel")]
      if !self.cel.is_empty() {
        let cel_ctx = ProgramsExecutionCtx {
          programs: &self.cel,
          value: val,
          ctx,
        };

        is_valid &= cel_ctx.execute_programs()?;
      }
    } else if self.required {
      handle_violation!(Required, "is required".to_string());
    }

    Ok(is_valid)
  }

  fn as_proto_option(&self) -> Option<ProtoOption> {
    Some(self.clone().into())
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

    set_options!(lt, lte, gt, gte);

    #[cfg(all(feature = "chrono", any(feature = "std", feature = "chrono-wasm")))]
    {
      rules
        .maybe_set("within", validator.within)
        .set_boolean("lt_now", validator.lt_now)
        .set_boolean("gt_now", validator.gt_now);
    }

    rules.maybe_set("const", validator.const_);

    let mut outer_rules = OptionMessageBuilder::new();

    if !rules.is_empty() {
      outer_rules.set("timestamp", OptionValue::Message(rules.into()));
    }

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
