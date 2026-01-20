mod builder;
pub use builder::DurationValidatorBuilder;

use proto_types::Duration;

use super::*;

#[non_exhaustive]
#[derive(Clone, Debug, Default)]
pub struct DurationValidator {
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  pub cel: Vec<CelProgram>,

  pub ignore: Ignore,

  /// Specifies that the field must be set in order to be valid.
  pub required: bool,

  /// Specifies that only the values in this list will be considered valid for this field.
  pub in_: Option<SortedList<Duration>>,

  /// Specifies that the values in this list will be considered NOT valid for this field.
  pub not_in: Option<SortedList<Duration>>,

  /// Specifies that only this specific value will be considered valid for this field.
  pub const_: Option<Duration>,

  /// Specifies that the value must be smaller than the indicated amount in order to pass validation.
  pub lt: Option<Duration>,

  /// Specifies that the value must be equal to or smaller than the indicated amount in order to pass validation.
  pub lte: Option<Duration>,

  /// Specifies that the value must be greater than the indicated amount in order to pass validation.
  pub gt: Option<Duration>,

  /// Specifies that the value must be equal to or greater than the indicated amount in order to pass validation.
  pub gte: Option<Duration>,

  pub error_messages: Option<ErrorMessages<DurationViolation>>,
}

impl Validator<Duration> for DurationValidator {
  type Target = Duration;

  impl_testing_methods!();

  fn check_consistency(&self) -> Result<(), Vec<ConsistencyError>> {
    let mut errors = Vec::new();

    macro_rules! check_prop_some {
      ($($id:ident),*) => {
        $(self.$id.is_some()) ||*
      };
    }

    if self.const_.is_some()
      && (!self.cel.is_empty() || check_prop_some!(in_, not_in, lt, lte, gt, gte))
    {
      errors.push(ConsistencyError::ConstWithOtherRules);
    }

    #[cfg(feature = "cel")]
    if let Err(e) = self.check_cel_programs() {
      errors.extend(e.into_iter().map(ConsistencyError::from));
    }

    if let Err(e) = check_list_rules(self.in_.as_ref(), self.not_in.as_ref()) {
      errors.push(e.into());
    }

    if let Err(e) = check_comparable_rules(self.lt, self.lte, self.gt, self.gte) {
      errors.push(e);
    }

    if errors.is_empty() {
      Ok(())
    } else {
      Err(errors)
    }
  }

  fn validate_core<V>(&self, ctx: &mut ValidationCtx, val: Option<&V>) -> bool
  where
    V: Borrow<Self::Target> + ?Sized,
  {
    handle_ignore_always!(&self.ignore);

    let mut is_valid = true;

    if let Some(val) = val {
      let val = *val.borrow();

      macro_rules! handle_violation {
        ($id:ident, $default:expr) => {
          ctx.add_duration_violation(
            DurationViolation::$id,
            self
              .error_messages
              .as_deref()
              .and_then(|map| map.get(&DurationViolation::$id))
              .map(|m| Cow::Borrowed(m.as_ref()))
              .unwrap_or_else(|| Cow::Owned($default)),
          );

          if ctx.fail_fast {
            return false;
          } else {
            is_valid = false;
          }
        };
      }

      if let Some(const_val) = self.const_ {
        if val != const_val {
          handle_violation!(Const, format!("must be equal to {const_val}"));
        }

        // Using `const` implies no other rules
        return is_valid;
      }

      if let Some(gt) = self.gt
        && val <= gt
      {
        handle_violation!(Gt, format!("must be longer than {gt}"));
      }

      if let Some(gte) = self.gte
        && val < gte
      {
        handle_violation!(Gte, format!("must be longer than or equal to {gte}"));
      }

      if let Some(lt) = self.lt
        && val >= lt
      {
        handle_violation!(Lt, format!("must be shorter than {lt}"));
      }

      if let Some(lte) = self.lte
        && val > lte
      {
        handle_violation!(Lte, format!("must be shorter than or equal to {lte}"));
      }

      if let Some(allowed_list) = &self.in_
        && !allowed_list.contains(&val)
      {
        handle_violation!(
          In,
          format!(
            "must be one of these values: {}",
            Duration::format_list(allowed_list)
          )
        );
      }

      if let Some(forbidden_list) = &self.not_in
        && forbidden_list.items.contains(&val)
      {
        handle_violation!(
          NotIn,
          format!(
            "must be one of these values: {}",
            Duration::format_list(forbidden_list)
          )
        );
      }

      #[cfg(feature = "cel")]
      if !self.cel.is_empty() {
        let cel_ctx = ProgramsExecutionCtx {
          programs: &self.cel,
          value: val,
          ctx,
        };

        is_valid = cel_ctx.execute_programs();
      }
    } else if self.required {
      ctx.add_required_violation();
      is_valid = false;
    }

    is_valid
  }

  fn as_proto_option(&self) -> Option<ProtoOption> {
    Some(self.clone().into())
  }
}

impl From<DurationValidator> for ProtoOption {
  fn from(validator: DurationValidator) -> Self {
    let mut rules = OptionMessageBuilder::new();

    rules
      .maybe_set("const", validator.const_)
      .maybe_set("lt", validator.lt)
      .maybe_set("lte", validator.lte)
      .maybe_set("gt", validator.gt)
      .maybe_set("gte", validator.gte)
      .maybe_set(
        "in",
        validator
          .in_
          .map(|list| OptionValue::new_list(list)),
      )
      .maybe_set(
        "not_in",
        validator
          .not_in
          .map(|list| OptionValue::new_list(list)),
      );

    let mut outer_rules = OptionMessageBuilder::new();

    if !rules.is_empty() {
      outer_rules.set("duration", OptionValue::Message(rules.build()));
    }

    outer_rules
      .add_cel_options(validator.cel)
      .set_required(validator.required)
      .set_ignore(validator.ignore);

    Self {
      name: "(buf.validate.field)".into(),
      value: OptionValue::Message(outer_rules.build()),
    }
  }
}
