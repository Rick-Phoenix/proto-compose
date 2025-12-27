pub mod builder;
pub use builder::DurationValidatorBuilder;
use builder::state::State;

use proto_types::Duration;

use super::*;

impl_validator!(DurationValidator, Duration);

#[derive(Clone, Debug)]
pub struct DurationValidator {
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  pub cel: Vec<&'static CelProgram>,

  pub ignore: Ignore,

  /// Specifies that the field must be set in order to be valid.
  pub required: bool,

  /// Specifies that only the values in this list will be considered valid for this field.
  pub in_: Option<&'static StaticLookup<Duration>>,

  /// Specifies that the values in this list will be considered NOT valid for this field.
  pub not_in: Option<&'static StaticLookup<Duration>>,

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

  fn make_unique_store<'a>(&self, cap: usize) -> Self::UniqueStore<'a> {
    CopyHybridStore::default_with_capacity(cap)
  }

  impl_testing_methods!();

  #[cfg(feature = "testing")]
  fn check_consistency(&self) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    #[cfg(feature = "cel")]
    if let Err(e) = self.check_cel_programs() {
      errors.extend(e.into_iter().map(|e| e.to_string()));
    }

    if let Err(e) = check_list_rules(self.in_, self.not_in) {
      errors.push(e.to_string());
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

    let mut violations_agg = Violations::new();
    let violations = &mut violations_agg;

    if let Some(&val) = val {
      if let Some(const_val) = self.const_
        && val != const_val
      {
        violations.add(
          field_context,
          parent_elements,
          &DURATION_CONST_VIOLATION,
          &format!("must be equal to {const_val}"),
        );
      }

      if let Some(gt) = self.gt
        && val <= gt
      {
        violations.add(
          field_context,
          parent_elements,
          &DURATION_GT_VIOLATION,
          &format!("must be longer than {gt}"),
        );
      }

      if let Some(gte) = self.gte
        && val < gte
      {
        violations.add(
          field_context,
          parent_elements,
          &DURATION_GTE_VIOLATION,
          &format!("must be longer than or equal to {gte}"),
        );
      }

      if let Some(lt) = self.lt
        && val >= lt
      {
        violations.add(
          field_context,
          parent_elements,
          &DURATION_LT_VIOLATION,
          &format!("must be shorter than {lt}"),
        );
      }

      if let Some(lte) = self.lte
        && val > lte
      {
        violations.add(
          field_context,
          parent_elements,
          &DURATION_LTE_VIOLATION,
          &format!("must be shorter than or equal to {lte}"),
        );
      }

      if let Some(allowed_list) = self.in_
        && !val.is_in(&allowed_list.items)
      {
        let err = ["must be one of these values: ", &allowed_list.items_str].concat();

        violations.add(field_context, parent_elements, &DURATION_IN_VIOLATION, &err);
      }

      if let Some(forbidden_list) = self.not_in
        && val.is_in(&forbidden_list.items)
      {
        let err = ["cannot be one of these values: ", &forbidden_list.items_str].concat();

        violations.add(
          field_context,
          parent_elements,
          &DURATION_NOT_IN_VIOLATION,
          &err,
        );
      }

      #[cfg(feature = "cel")]
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

impl From<DurationValidator> for ProtoOption {
  fn from(validator: DurationValidator) -> Self {
    let mut rules: OptionValueList = Vec::new();

    if let Some(const_val) = validator.const_ {
      rules.push((CONST_.clone(), OptionValue::Duration(const_val)));
    }

    insert_option!(validator, rules, lt);
    insert_option!(validator, rules, lte);
    insert_option!(validator, rules, gt);
    insert_option!(validator, rules, gte);

    if let Some(allowed_list) = &validator.in_ {
      rules.push((
        IN_.clone(),
        OptionValue::new_list(allowed_list.items.iter()),
      ));
    }

    if let Some(forbidden_list) = &validator.not_in {
      rules.push((
        NOT_IN.clone(),
        OptionValue::new_list(forbidden_list.items.iter()),
      ));
    }

    let mut outer_rules: OptionValueList = vec![];

    outer_rules.push((DURATION.clone(), OptionValue::Message(rules.into())));

    insert_cel_rules!(validator, outer_rules);
    insert_boolean_option!(validator, outer_rules, required);

    if !validator.ignore.is_default() {
      outer_rules.push((IGNORE.clone(), validator.ignore.into()))
    }

    Self {
      name: BUF_VALIDATE_FIELD.clone(),
      value: OptionValue::Message(outer_rules.into()),
    }
  }
}
