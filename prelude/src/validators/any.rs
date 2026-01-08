pub mod builder;
pub use builder::AnyValidatorBuilder;
use builder::state::State;

use proto_types::Any;

use super::*;

impl_validator!(AnyValidator, Any);

#[derive(Clone, Debug)]
pub struct AnyValidator {
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  pub cel: Vec<CelProgram>,

  pub ignore: Ignore,

  /// Specifies that the field must be set in order to be valid.
  pub required: bool,

  /// Specifies that only the values in this list will be considered valid for this field.
  pub in_: Option<StaticLookup<&'static str>>,

  /// Specifies that the values in this list will be considered NOT valid for this field.
  pub not_in: Option<StaticLookup<&'static str>>,
}

impl Validator<Any> for AnyValidator {
  type Target = Any;
  type UniqueStore<'a>
    = LinearRefStore<'a, Any>
  where
    Self: 'a;

  #[inline]
  fn make_unique_store<'a>(&self, cap: usize) -> Self::UniqueStore<'a> {
    LinearRefStore::default_with_capacity(cap)
  }

  impl_testing_methods!();

  fn check_consistency(&self) -> Result<(), Vec<ConsistencyError>> {
    let mut errors = Vec::new();

    #[cfg(feature = "cel")]
    if let Err(e) = self.check_cel_programs() {
      errors.extend(e.into_iter().map(ConsistencyError::from));
    }

    if let Err(e) = check_list_rules(self.in_.as_ref(), self.not_in.as_ref()) {
      errors.push(e.into());
    }

    if errors.is_empty() {
      Ok(())
    } else {
      Err(errors)
    }
  }

  fn validate(&self, ctx: &mut ValidationCtx, val: Option<&Self::Target>) {
    handle_ignore_always!(&self.ignore);
    handle_ignore_if_zero_value!(&self.ignore, val.is_none_or(|v| v.is_default()));

    if let Some(val) = val {
      if let Some(allowed_list) = &self.in_
        && !allowed_list
          .items
          .contains(&val.type_url.as_str())
      {
        let err = [
          "must have one of these type URLs: ",
          &allowed_list.items_str,
        ]
        .concat();

        ctx.add_violation(&ANY_IN_VIOLATION, &err);
      }

      if let Some(forbidden_list) = &self.not_in
        && forbidden_list
          .items
          .contains(&val.type_url.as_str())
      {
        let err = [
          "cannot have one of these type URLs: ",
          &forbidden_list.items_str,
        ]
        .concat();

        ctx.add_violation(&ANY_NOT_IN_VIOLATION, &err);
      }

      #[cfg(feature = "cel")]
      if !self.cel.is_empty() {
        let ctx = ProgramsExecutionCtx {
          programs: &self.cel,
          value: val.clone(),
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

impl From<AnyValidator> for ProtoOption {
  fn from(validator: AnyValidator) -> Self {
    let mut rules = OptionMessageBuilder::new();

    rules
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

    outer_rules.set(ANY.clone(), OptionValue::Message(rules.build()));

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
