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
    Ok(())
  }

  #[cfg(feature = "cel")]
  #[doc(hidden)]
  fn check_cel_programs_with(&self, _val: Self::Target) -> Result<(), Vec<CelError>> {
    // No CEL rules in this one
    Ok(())
  }

  fn validate_core<V>(&self, ctx: &mut ValidationCtx, val: Option<&V>) -> bool
  where
    V: Borrow<Self::Target> + ?Sized,
  {
    handle_ignore_always!(&self.ignore);
    handle_ignore_if_zero_value!(&self.ignore, val.is_none_or(|v| v.borrow().is_default()));

    let mut is_valid = true;

    if let Some(val) = val {
      let val = *val.borrow();

      if let Some(const_val) = self.const_
        && val != const_val
      {
        ctx.add_violation(
          ViolationKind::Bool(BoolViolation::Const),
          format!("must be {const_val}"),
        );
        is_valid = false;
      }
    } else if self.required {
      ctx.add_required_violation();
      is_valid = false;
    }

    is_valid
  }

  fn into_proto_option(self) -> Option<ProtoOption> {
    Some(self.into())
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
}

impl From<BoolValidator> for ProtoOption {
  fn from(validator: BoolValidator) -> Self {
    let mut rules = OptionMessageBuilder::new();

    rules.maybe_set("const", validator.const_);

    let mut outer_rules = OptionMessageBuilder::new();

    outer_rules.set("bool", OptionValue::Message(rules.build()));

    outer_rules
      .set_required(validator.required)
      .set_ignore(validator.ignore);

    Self {
      name: "(buf.validate.field)".into(),
      value: OptionValue::Message(outer_rules.build()),
    }
  }
}
