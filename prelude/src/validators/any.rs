use any_validator_builder::{IsUnset, SetIgnore, State};
use bon::Builder;
use proto_types::Any;

use super::*;

impl_validator!(AnyValidator, Any);
impl_into_option!(AnyValidator);

impl Validator<Any> for AnyValidator {
  type Target = Any;

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
        violations.add(field_context, parent_elements, &ANY_IN_VIOLATION, &format!("must have one of these type URLs: {}", format_list(allowed_list.into_iter())));
      }

      if let Some(forbidden_list) = &self.not_in && <&Any>::is_in(forbidden_list, val) {
        violations.add(field_context, parent_elements, &ANY_NOT_IN_VIOLATION, &format!("cannot have one of these type URLs: {}", format_list(forbidden_list.into_iter())));
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
  /// Specifies that the given `google.protobuf.Any` message must have a type URL that is contained in this list.
  #[builder(into)]
  pub in_: Option<ItemLookup<'static, &'static str>>,
  /// Specifies that the given `google.protobuf.Any` message must have a type URL that is NOT contained in this list.
  #[builder(into)]
  pub not_in: Option<ItemLookup<'static, &'static str>>,
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

    insert_list_option!(validator, rules, in_);
    insert_list_option!(validator, rules, not_in);

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
