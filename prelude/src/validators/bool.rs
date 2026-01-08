pub mod builder;
pub use builder::BoolValidatorBuilder;
use builder::state::State;

use super::*;

impl_validator!(BoolValidator, bool);
impl_proto_type!(bool, Bool);

impl Validator<bool> for BoolValidator {
  type Target = bool;
  type UniqueStore<'a>
    = CopyHybridStore<bool>
  where
    Self: 'a;

  #[doc(hidden)]
  fn cel_rules(&self) -> Vec<CelRule> {
    Vec::new()
  }

  #[inline]
  #[doc(hidden)]
  fn make_unique_store<'a>(&self, _: usize) -> Self::UniqueStore<'a> {
    // This is likely to never be used in the first place, but
    // uniqueness checks would fail after more than 2 elements anyway
    CopyHybridStore::default_with_capacity(2)
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

  fn validate(&self, ctx: &mut ValidationCtx, val: Option<&Self::Target>) {
    handle_ignore_always!(&self.ignore);
    handle_ignore_if_zero_value!(&self.ignore, val.is_none_or(|v| v.is_default()));

    if let Some(&val) = val {
      if let Some(const_val) = self.const_
        && val != const_val
      {
        ctx.add_violation(&BOOL_CONST_VIOLATION, &format!("must be {const_val}"));
      }
    } else if self.required {
      ctx.add_required_violation();
    }
  }
}

#[derive(Clone, Debug)]
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

    if let Some(value) = validator.const_ {
      rules.set(CONST_.clone(), OptionValue::Bool(value));
    }

    let mut outer_rules = OptionMessageBuilder::new();

    outer_rules.set(BOOL.clone(), OptionValue::Message(rules.build()));

    outer_rules
      .set_required(validator.required)
      .set_ignore(validator.ignore);

    Self {
      name: BUF_VALIDATE_FIELD.clone(),
      value: OptionValue::Message(outer_rules.build()),
    }
  }
}
