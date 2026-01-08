pub mod builder;
pub use builder::DurationValidatorBuilder;
use builder::state::State;

use proto_types::Duration;

use super::*;

impl_validator!(DurationValidator, Duration);

#[derive(Clone, Debug)]
pub struct DurationValidator {
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  pub cel: Vec<CelProgram>,

  pub ignore: Ignore,

  /// Specifies that the field must be set in order to be valid.
  pub required: bool,

  /// Specifies that only the values in this list will be considered valid for this field.
  pub in_: Option<StaticLookup<Duration>>,

  /// Specifies that the values in this list will be considered NOT valid for this field.
  pub not_in: Option<StaticLookup<Duration>>,

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
}

impl Validator<Duration> for DurationValidator {
  type Target = Duration;
  type UniqueStore<'a>
    = CopyHybridStore<Duration>
  where
    Self: 'a;

  #[inline]
  fn make_unique_store<'a>(&self, cap: usize) -> Self::UniqueStore<'a> {
    CopyHybridStore::default_with_capacity(cap)
  }

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

  fn validate(&self, ctx: &mut ValidationCtx, val: Option<&Self::Target>) {
    handle_ignore_always!(&self.ignore);

    if let Some(&val) = val {
      if let Some(const_val) = self.const_ {
        if val != const_val {
          ctx.add_violation(
            &DURATION_CONST_VIOLATION,
            &format!("must be equal to {const_val}"),
          );
        }

        // Using `const` implies no other rules
        return;
      }

      if let Some(gt) = self.gt
        && val <= gt
      {
        ctx.add_violation(&DURATION_GT_VIOLATION, &format!("must be longer than {gt}"));
      }

      if let Some(gte) = self.gte
        && val < gte
      {
        ctx.add_violation(
          &DURATION_GTE_VIOLATION,
          &format!("must be longer than or equal to {gte}"),
        );
      }

      if let Some(lt) = self.lt
        && val >= lt
      {
        ctx.add_violation(
          &DURATION_LT_VIOLATION,
          &format!("must be shorter than {lt}"),
        );
      }

      if let Some(lte) = self.lte
        && val > lte
      {
        ctx.add_violation(
          &DURATION_LTE_VIOLATION,
          &format!("must be shorter than or equal to {lte}"),
        );
      }

      if let Some(allowed_list) = &self.in_
        && !allowed_list.items.contains(&val)
      {
        let err = ["must be one of these values: ", &allowed_list.items_str].concat();

        ctx.add_violation(&DURATION_IN_VIOLATION, &err);
      }

      if let Some(forbidden_list) = &self.not_in
        && forbidden_list.items.contains(&val)
      {
        let err = ["cannot be one of these values: ", &forbidden_list.items_str].concat();

        ctx.add_violation(&DURATION_NOT_IN_VIOLATION, &err);
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

impl From<DurationValidator> for ProtoOption {
  fn from(validator: DurationValidator) -> Self {
    let mut rules = OptionMessageBuilder::new();

    rules.maybe_set(&CONST_, validator.const_);

    rules
      .maybe_set(&LT, validator.lt)
      .maybe_set(&LTE, validator.lte)
      .maybe_set(&GT, validator.gt)
      .maybe_set(&GTE, validator.gte)
      .maybe_set(
        &IN_,
        validator
          .in_
          .map(|list| OptionValue::new_list(list.items)),
      )
      .maybe_set(
        &NOT_IN,
        validator
          .not_in
          .map(|list| OptionValue::new_list(list.items)),
      );

    let mut outer_rules = OptionMessageBuilder::new();

    outer_rules.set(DURATION.clone(), OptionValue::Message(rules.build()));

    outer_rules
      .add_cel_options(validator.cel)
      .set_required(validator.required)
      .set_ignore(validator.ignore);

    Self {
      name: BUF_VALIDATE_FIELD.clone(),
      value: OptionValue::Message(outer_rules.build()),
    }
  }
}
