use any_validator_builder::{IsUnset, SetIgnore, State};
use bon::Builder;
use proto_types::Any;

use super::*;

impl_validator!(AnyValidator, Any);
impl_into_option!(AnyValidator);

impl Validator<Any> for AnyValidator {
  type Target = Any;

  impl_testing_methods!();

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

    if let Some(val) = val {
      if let Some(allowed_list) = &self.in_ && !<&Any>::is_in(allowed_list, val) {
        violations.add(field_context, parent_elements, &ANY_IN_VIOLATION, &format!("must have one of these type URLs: {}", format_list(allowed_list.iter())));
      }

      if let Some(forbidden_list) = &self.not_in && <&Any>::is_in(forbidden_list, val) {
        violations.add(field_context, parent_elements, &ANY_NOT_IN_VIOLATION, &format!("cannot have one of these type URLs: {}", format_list(forbidden_list.iter())));
      }

      if !self.cel.is_empty() {
        let ctx = ProgramsExecutionCtx {
          programs: &self.cel,
          value: val.clone(),
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

#[derive(Clone, Debug, Builder)]
#[builder(derive(Clone))]
pub struct AnyValidator {
  /// Specifies that only the values in this list will be considered valid for this field.
  pub in_: Option<&'static ItemLookup<&'static str>>,
  /// Specifies that the values in this list will be considered NOT valid for this field.
  pub not_in: Option<&'static ItemLookup<&'static str>>,
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  #[builder(default, with = |programs: impl IntoIterator<Item = &'static LazyLock<CelProgram>>| collect_programs(programs))]
  pub cel: Vec<&'static CelProgram>,
  /// Specifies that the field must be set in order to be valid.
  #[builder(default, with = || true)]
  pub required: bool,
  #[builder(setters(vis = "", name = ignore))]
  pub ignore: Option<Ignore>,
}

impl<S: State> AnyValidatorBuilder<S>
where
  S::Ignore: IsUnset,
{
  /// Rules set for this field will always be ignored.
  pub fn ignore_always(self) -> AnyValidatorBuilder<SetIgnore<S>> {
    self.ignore(Ignore::Always)
  }
}

impl From<AnyValidator> for ProtoOption {
  fn from(validator: AnyValidator) -> Self {
    let mut rules: OptionValueList = Vec::new();

    if let Some(allowed_list) = &validator.in_ {
      rules.push((IN_.clone(), OptionValue::new_list(allowed_list.iter())));
    }

    if let Some(forbidden_list) = &validator.not_in {
      rules.push((NOT_IN.clone(), OptionValue::new_list(forbidden_list.iter())));
    }

    let mut outer_rules: OptionValueList = vec![];

    outer_rules.push((ANY.clone(), OptionValue::Message(rules.into())));

    insert_cel_rules!(validator, outer_rules);
    insert_boolean_option!(validator, outer_rules, required);
    insert_option!(validator, outer_rules, ignore);

    ProtoOption {
      name: BUF_VALIDATE_FIELD.clone(),
      value: OptionValue::Message(outer_rules.into()),
    }
  }
}
