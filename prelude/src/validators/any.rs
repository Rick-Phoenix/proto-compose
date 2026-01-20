mod builder;
pub use builder::AnyValidatorBuilder;

use proto_types::Any;

use super::*;

#[non_exhaustive]
#[derive(Clone, Debug, Default)]
pub struct AnyValidator {
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  pub cel: Vec<CelProgram>,

  pub ignore: Ignore,

  /// Specifies that the field must be set in order to be valid.
  pub required: bool,

  /// Specifies that only the values in this list will be considered valid for this field.
  pub in_: Option<SortedList<SharedStr>>,

  /// Specifies that the values in this list will be considered NOT valid for this field.
  pub not_in: Option<SortedList<SharedStr>>,

  pub error_messages: Option<ErrorMessages<AnyViolation>>,
}

impl Validator<Any> for AnyValidator {
  type Target = Any;

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

  fn validate_core<V>(&self, ctx: &mut ValidationCtx, val: Option<&V>) -> bool
  where
    V: Borrow<Self::Target> + ?Sized,
  {
    handle_ignore_always!(&self.ignore);
    handle_ignore_if_zero_value!(&self.ignore, val.is_none_or(|v| v.borrow().is_default()));

    let mut is_valid = true;

    if let Some(val) = val {
      let val = val.borrow();

      macro_rules! handle_violation {
        ($id:ident, $default:expr) => {
          ctx.add_any_violation(
            AnyViolation::$id,
            self
              .error_messages
              .as_deref()
              .and_then(|map| map.get(&AnyViolation::$id))
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

      if let Some(allowed_list) = &self.in_
        && !allowed_list.contains(val.type_url.as_str())
      {
        handle_violation!(
          In,
          format!(
            "must have one of these type URLs: {}",
            SharedStr::format_list(allowed_list)
          )
        );
      }

      if let Some(forbidden_list) = &self.not_in
        && forbidden_list.contains(val.type_url.as_str())
      {
        handle_violation!(
          NotIn,
          format!(
            "cannot have one of these type URLs: {}",
            SharedStr::format_list(forbidden_list)
          )
        );
      }

      #[cfg(feature = "cel")]
      if !self.cel.is_empty() {
        let cel_ctx = ProgramsExecutionCtx {
          programs: &self.cel,
          value: val.clone(),
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

impl From<AnyValidator> for ProtoOption {
  fn from(validator: AnyValidator) -> Self {
    let mut rules = OptionMessageBuilder::new();

    rules
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
      outer_rules.set("any", OptionValue::Message(rules.build()));
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
