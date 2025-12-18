use bon::Builder;
use bool_validator_builder::State;

use super::*;

impl_validator!(BoolValidator, bool);
impl_into_option!(BoolValidator);

impl Validator<bool> for BoolValidator {
  type Target = bool;

  fn validate(
    &self,
    field_context: &FieldContext,
    parent_elements: &mut Vec<FieldPathElement>,
    val: Option<&Self::Target>,
  ) -> Result<(), Violations> {
    handle_ignore_always!(&self.ignore);
    handle_ignore_if_zero_value!(&self.ignore, val.is_none_or(|v| v.is_default()));

    let mut violations_agg = Violations::new();
    let violations = &mut violations_agg;

    if let Some(&val) = val {
      if let Some(const_val) = self.const_ && val != const_val {
        violations.add(field_context, parent_elements, &BOOL_CONST_VIOLATION, &format!("must be {const_val}"));
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

impl_ignore!(BoolValidatorBuilder);

#[derive(Clone, Debug, Builder)]
pub struct BoolValidator {
  /// Specifies that only this specific value will be considered valid for this field.
  pub const_: Option<bool>,
  #[builder(default, with = || true)]
  /// Specifies that the field must be set in order to be valid.
  pub required: bool,
  #[builder(setters(vis = "", name = ignore))]
  pub ignore: Option<Ignore>,
}

impl From<BoolValidator> for ProtoOption {
  fn from(validator: BoolValidator) -> Self {
    let mut rules: OptionValueList = Vec::new();

    insert_option!(validator, rules, const_);

    let mut outer_rules: OptionValueList = vec![];

    outer_rules.push((BOOL.clone(), OptionValue::Message(rules.into())));

    insert_boolean_option!(validator, outer_rules, required);
    insert_option!(validator, outer_rules, ignore);

    ProtoOption {
      name: BUF_VALIDATE_FIELD.clone(),
      value: OptionValue::Message(outer_rules.into()),
    }
  }
}
