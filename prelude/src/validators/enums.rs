mod builder;
pub use builder::EnumValidatorBuilder;

use super::*;

#[non_exhaustive]
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

impl<T: ProtoEnum> Default for EnumValidator<T> {
  #[inline]
  fn default() -> Self {
    Self {
      cel: Default::default(),
      ignore: Default::default(),
      _enum: PhantomData,
      defined_only: Default::default(),
      required: Default::default(),
      in_: Default::default(),
      not_in: Default::default(),
      const_: Default::default(),
    }
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

    if let Some(const_val) = &self.const_
      && T::try_from(*const_val).is_err()
    {
      errors.push(ConsistencyError::ContradictoryInput(format!("The `const` value for the enum `{}` is {const_val} but this number is not among its variants.", T::proto_name())));
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

  fn validate(&self, ctx: &mut ValidationCtx, val: Option<&Self::Target>) -> bool {
    handle_ignore_always!(&self.ignore);
    handle_ignore_if_zero_value!(&self.ignore, val.is_none_or(|v| v.is_default()));

    let mut is_valid = true;

    if self.required && val.is_none_or(|v| *v == 0) {
      ctx.add_required_violation();
      return false;
    }

    if let Some(&val) = val {
      if let Some(const_val) = self.const_ {
        if val != const_val {
          ctx.add_violation(
            ENUM_CONST_VIOLATION,
            &format!("must be equal to {const_val}"),
          );

          is_valid = false;
        }

        // Using `const` implies no other rules
        return is_valid;
      }

      if self.defined_only && T::try_from(val).is_err() {
        ctx.add_violation(ENUM_DEFINED_ONLY_VIOLATION, "must be a known enum value");
        handle_violation!(is_valid, ctx);
      }

      if let Some(allowed_list) = &self.in_
        && !allowed_list.items.contains(&val)
      {
        let err = ["must be one of these values: ", &allowed_list.items_str].concat();

        ctx.add_violation(ENUM_IN_VIOLATION, &err);
        handle_violation!(is_valid, ctx);
      }

      if let Some(forbidden_list) = &self.not_in
        && forbidden_list.items.contains(&val)
      {
        let err = ["cannot be one of these values: ", &forbidden_list.items_str].concat();

        ctx.add_violation(ENUM_NOT_IN_VIOLATION, &err);
        handle_violation!(is_valid, ctx);
      }

      #[cfg(feature = "cel")]
      if !self.cel.is_empty() {
        let cel_ctx = ProgramsExecutionCtx {
          programs: &self.cel,
          value: val,
          ctx,
        };

        is_valid = cel_ctx.execute_programs();
      }
    }

    is_valid
  }
}

impl<T: ProtoEnum> From<EnumValidator<T>> for ProtoOption {
  fn from(validator: EnumValidator<T>) -> Self {
    let mut rules = OptionMessageBuilder::new();

    rules
      .maybe_set("const", validator.const_)
      .set_boolean("defined_only", validator.defined_only)
      .maybe_set(
        "in",
        validator
          .in_
          .map(|list| OptionValue::new_list(list.items)),
      )
      .maybe_set(
        "not_in",
        validator
          .not_in
          .map(|list| OptionValue::new_list(list.items)),
      );

    let mut outer_rules = OptionMessageBuilder::new();

    outer_rules.set("enum", OptionValue::Message(rules.into()));

    outer_rules
      .add_cel_options(validator.cel)
      .set_required(validator.required)
      .set_ignore(validator.ignore);

    Self {
      name: "(buf.validate.field)".into(),
      value: OptionValue::Message(outer_rules.into()),
    }
  }
}
