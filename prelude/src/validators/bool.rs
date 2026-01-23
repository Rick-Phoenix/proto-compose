mod builder;
pub use builder::BoolValidatorBuilder;

use super::*;

impl_proto_type!(bool, Bool);
impl_proto_map_key!(bool, Bool);

impl Validator<bool> for BoolValidator {
  type Target = bool;

  #[cfg(feature = "cel")]
  fn check_cel_programs(&self) -> Result<(), Vec<CelError>> {
    self.check_cel_programs_with(false)
  }

  #[doc(hidden)]
  fn cel_rules(&self) -> Vec<CelRule> {
    Vec::new()
  }

  #[inline]
  #[doc(hidden)]
  fn check_consistency(&self) -> Result<(), Vec<ConsistencyError>> {
    let mut errors = Vec::new();

    if let Some(custom_messages) = self.error_messages.as_deref() {
      let mut unused_messages: Vec<String> = Vec::new();

      for key in custom_messages.keys() {
        let is_used = match key {
          BoolViolation::Required => self.required,
          BoolViolation::Const => self.const_.is_some(),
          _ => true,
        };

        if !is_used {
          unused_messages.push(format!("{key:?}"));
        }
      }

      if !unused_messages.is_empty() {
        errors.push(ConsistencyError::UnusedCustomMessages(unused_messages));
      }
    }

    if errors.is_empty() {
      Ok(())
    } else {
      Err(errors)
    }
  }

  #[cfg(feature = "cel")]
  #[doc(hidden)]
  fn check_cel_programs_with(&self, _val: Self::Target) -> Result<(), Vec<CelError>> {
    // No CEL rules in this one
    Ok(())
  }

  fn validate_core<V>(&self, ctx: &mut ValidationCtx, val: Option<&V>) -> ValidatorResult
  where
    V: Borrow<Self::Target> + ?Sized,
  {
    handle_ignore_always!(&self.ignore);
    handle_ignore_if_zero_value!(&self.ignore, val.is_none_or(|v| v.borrow().is_default()));

    let mut is_valid = IsValid::Yes;

    macro_rules! handle_violation {
      ($id:ident, $default:expr) => {
        is_valid &= ctx.add_violation(
          ViolationKind::Bool(BoolViolation::$id),
          self
            .error_messages
            .as_deref()
            .and_then(|map| map.get(&BoolViolation::$id))
            .map(|m| Cow::Borrowed(m.as_ref()))
            .unwrap_or_else(|| Cow::Owned($default)),
        )?;
      };
    }

    if self.required && val.is_none_or(|v| !v.borrow()) {
      handle_violation!(Required, "is required".to_string());
      return Ok(is_valid);
    }

    if let Some(val) = val {
      let val = *val.borrow();

      if let Some(const_val) = self.const_
        && val != const_val
      {
        handle_violation!(Const, format!("must be {const_val}"));
      }
    }

    Ok(is_valid)
  }

  fn schema(&self) -> Option<ValidatorSchema> {
    Some(ValidatorSchema {
      schema: self.clone().into(),
      cel_rules: self.cel_rules(),
      imports: vec!["buf/validate/validate.proto".into()],
    })
  }
}

#[non_exhaustive]
#[derive(Clone, Debug, Default)]
pub struct BoolValidator {
  /// Specifies that only this specific value will be considered valid for this field.
  pub const_: Option<bool>,
  /// Specifies that the field must be set in order to be valid.
  pub required: bool,
  pub ignore: Ignore,

  pub error_messages: Option<ErrorMessages<BoolViolation>>,
}

impl From<BoolValidator> for ProtoOption {
  fn from(validator: BoolValidator) -> Self {
    let mut rules = OptionMessageBuilder::new();

    rules.maybe_set("const", validator.const_);

    let mut outer_rules = OptionMessageBuilder::new();

    if !rules.is_empty() {
      outer_rules.set("bool", OptionValue::Message(rules.build()));
    }

    outer_rules
      .set_required(validator.required)
      .set_ignore(validator.ignore);

    Self {
      name: "(buf.validate.field)".into(),
      value: OptionValue::Message(outer_rules.build()),
    }
  }
}
