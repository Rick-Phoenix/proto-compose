pub mod builder;
pub use builder::EnumValidatorBuilder;
use builder::state::State;

use super::*;

#[derive(Clone, Debug)]
pub struct EnumValidator<T: ProtoEnum> {
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  pub cel: Vec<CelProgram>,

  pub ignore: Ignore,

  _enum: PhantomData<T>,

  /// Marks that this field will only accept values that are defined in the enum that it's referring to.
  pub defined_only: bool,

  /// Specifies that the field must be set in order to be valid.
  pub required: bool,

  /// Specifies that only the values in this list will be considered valid for this field.
  pub in_: Option<StaticLookup<i32>>,

  /// Specifies that the values in this list will be considered NOT valid for this field.
  pub not_in: Option<StaticLookup<i32>>,

  /// Specifies that only this specific value will be considered valid for this field.
  pub const_: Option<i32>,
}

impl<T: ProtoEnum, S: State> ValidatorBuilderFor<T> for EnumValidatorBuilder<T, S> {
  type Target = i32;
  type Validator = EnumValidator<T>;

  #[inline]
  #[doc(hidden)]
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

  #[inline]
  fn make_unique_store<'a>(&self, cap: usize) -> Self::UniqueStore<'a>
  where
    T: 'a,
  {
    CopyHybridStore::default_with_capacity(cap)
  }

  fn check_consistency(&self) -> Result<(), Vec<ConsistencyError>> {
    let mut errors = Vec::new();

    macro_rules! check_prop_some {
      ($($id:ident),*) => {
        $(self.$id.is_some()) ||*
      };
    }

    if self.const_.is_some()
      && (!self.cel.is_empty() || self.defined_only || check_prop_some!(in_, not_in))
    {
      errors.push(ConsistencyError::ConstWithOtherRules);
    }

    #[cfg(feature = "cel")]
    if let Err(e) = self.check_cel_programs() {
      errors.extend(e.into_iter().map(ConsistencyError::from));
    }

    if let Err(e) = check_list_rules(self.in_.as_ref(), self.not_in.as_ref()) {
      errors.push(e.into());
    }

    if let Some(in_list) = &self.in_ {
      for num in in_list.items.iter() {
        if T::try_from(*num).is_err() {
          errors.push(ConsistencyError::ContradictoryInput(format!(
            "Number {num} is in the allowed list but it does not belong to the enum {}",
            T::proto_name()
          )));
        }
      }
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

    if self.required && val.is_none_or(|v| *v == 0) {
      ctx.add_required_violation();
    }

    if let Some(&val) = val {
      if let Some(const_val) = self.const_ {
        if val != const_val {
          ctx.add_violation(
            &ENUM_CONST_VIOLATION,
            &format!("must be equal to {const_val}"),
          );
        }

        // Using `const` implies no other rules
        return;
      }

      if let Some(allowed_list) = &self.in_
        && !allowed_list.items.contains(&val)
      {
        let err = ["must be one of these values: ", &allowed_list.items_str].concat();

        ctx.add_violation(&ENUM_IN_VIOLATION, &err);
      }

      if let Some(forbidden_list) = &self.not_in
        && forbidden_list.items.contains(&val)
      {
        let err = ["cannot be one of these values: ", &forbidden_list.items_str].concat();

        ctx.add_violation(&ENUM_NOT_IN_VIOLATION, &err);
      }

      if self.defined_only && T::try_from(val).is_err() {
        ctx.add_violation(&ENUM_DEFINED_ONLY_VIOLATION, "must be a known enum value");
      }

      #[cfg(feature = "cel")]
      if !self.cel.is_empty() {
        let ctx = ProgramsExecutionCtx {
          programs: &self.cel,
          value: val,
          violations: ctx.violations,
          field_context: Some(&ctx.field_context),
          parent_elements: ctx.parent_elements,
        };

        ctx.execute_programs();
      }
    }
  }
}

impl<T: ProtoEnum> From<EnumValidator<T>> for ProtoOption {
  fn from(validator: EnumValidator<T>) -> Self {
    let mut rules: OptionValueList = Vec::new();

    if let Some(const_val) = validator.const_ {
      rules.push((CONST_.clone(), OptionValue::Int(i64::from(const_val))));
    }

    insert_boolean_option!(validator, rules, defined_only);

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
