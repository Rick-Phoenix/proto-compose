pub mod builder;
pub use builder::EnumValidatorBuilder;
use builder::state::State;

use super::*;

impl<T: ProtoEnum, S: State> ValidatorBuilderFor<T> for EnumValidatorBuilder<T, S> {
  type Target = i32;
  type Validator = EnumValidator<T>;

  fn build_validator(self) -> Self::Validator {
    self.build()
  }
}

impl<T: ProtoEnum> Validator<T> for EnumValidator<T> {
  type Target = i32;
  type UniqueStore<'a>
    = CopyHybridStore<i32>
  where
    Self: 'a;

  impl_testing_methods!();

  fn make_unique_store<'a>(&self, cap: usize) -> Self::UniqueStore<'a>
  where
    T: 'a,
  {
    CopyHybridStore::default_with_capacity(cap)
  }

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

    if let Some(in_list) = self.in_ {
      for num in in_list {
        if T::try_from(*num).is_err() {
          errors.push(format!(
            "Number {num} is in the allowed list but it does not belong to the enum {}",
            T::full_name()
          ));
        }
      }
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
    handle_ignore_if_zero_value!(&self.ignore, val.is_none_or(|v| v.is_default()));

    let mut violations_agg = Violations::new();
    let violations = &mut violations_agg;

    if let Some(&val) = val {
      if let Some(const_val) = self.const_
        && val != const_val
      {
        violations.add(
          field_context,
          parent_elements,
          &ENUM_CONST_VIOLATION,
          &format!("must be equal to {const_val}"),
        );
      }

      if let Some(allowed_list) = &self.in_
        && !protocheck_core::wrappers::EnumVariant::is_in(
          &protocheck_core::wrappers::EnumVariant(val),
          allowed_list,
        )
      {
        violations.add(
          field_context,
          parent_elements,
          &ENUM_IN_VIOLATION,
          &format!(
            "must be one of these values: {}",
            format_list(allowed_list.iter())
          ),
        );
      }

      if let Some(forbidden_list) = &self.not_in
        && protocheck_core::wrappers::EnumVariant::is_in(
          &protocheck_core::wrappers::EnumVariant(val),
          forbidden_list,
        )
      {
        violations.add(
          field_context,
          parent_elements,
          &ENUM_NOT_IN_VIOLATION,
          &format!(
            "cannot be one of these values: {}",
            format_list(forbidden_list.iter())
          ),
        );
      }

      if self.defined_only && T::try_from(val).is_err() {
        violations.add(
          field_context,
          parent_elements,
          &ENUM_DEFINED_ONLY_VIOLATION,
          "must be a known enum value",
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

#[derive(Clone, Debug)]
pub struct EnumValidator<T: ProtoEnum> {
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  pub cel: Vec<&'static CelProgram>,

  pub ignore: Ignore,

  _enum: PhantomData<T>,

  /// Marks that this field will only accept values that are defined in the enum that it's referring to.
  pub defined_only: bool,

  /// Specifies that the field must be set in order to be valid.
  pub required: bool,

  /// Specifies that only the values in this list will be considered valid for this field.
  pub in_: Option<&'static SortedList<i32>>,

  /// Specifies that the values in this list will be considered NOT valid for this field.
  pub not_in: Option<&'static SortedList<i32>>,

  /// Specifies that only this specific value will be considered valid for this field.
  pub const_: Option<i32>,
}

impl<T: ProtoEnum> From<EnumValidator<T>> for ProtoOption {
  fn from(validator: EnumValidator<T>) -> Self {
    let mut rules: OptionValueList = Vec::new();

    if let Some(const_val) = validator.const_ {
      rules.push((CONST_.clone(), OptionValue::Int(i64::from(const_val))));
    }

    insert_boolean_option!(validator, rules, defined_only);

    if let Some(allowed_list) = &validator.in_ {
      rules.push((IN_.clone(), OptionValue::new_list(allowed_list.iter())));
    }

    if let Some(forbidden_list) = &validator.not_in {
      rules.push((NOT_IN.clone(), OptionValue::new_list(forbidden_list.iter())));
    }

    let mut outer_rules: OptionValueList = vec![];

    outer_rules.push((ENUM.clone(), OptionValue::Message(rules.into())));

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
