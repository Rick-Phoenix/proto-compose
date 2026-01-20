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
  pub in_: Option<SortedList<i32>>,

  /// Specifies that the values in this list will be considered NOT valid for this field.
  pub not_in: Option<SortedList<i32>>,

  /// Specifies that only this specific value will be considered valid for this field.
  pub const_: Option<i32>,

  pub error_messages: Option<ErrorMessages<EnumViolation>>,
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
      error_messages: None,
    }
  }
}

impl<T: ProtoEnum> Validator<T> for EnumValidator<T> {
  type Target = i32;

  impl_testing_methods!();

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

  fn validate_core<V>(&self, ctx: &mut ValidationCtx, val: Option<&V>) -> bool
  where
    V: Borrow<Self::Target> + ?Sized,
  {
    handle_ignore_always!(&self.ignore);
    handle_ignore_if_zero_value!(&self.ignore, val.is_none_or(|v| v.borrow().is_default()));

    let mut is_valid = true;

    if self.required && val.is_none_or(|v| *v.borrow() == 0) {
      ctx.add_required_violation();
      return false;
    }

    if let Some(val) = val {
      let val = *val.borrow();

      macro_rules! handle_violation {
        ($id:ident, $default:expr) => {
          ctx.add_enum_violation(
            EnumViolation::$id,
            self
              .error_messages
              .as_deref()
              .and_then(|map| map.get(&EnumViolation::$id))
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

      if let Some(const_val) = self.const_ {
        if val != const_val {
          handle_violation!(Const, format!("must be equal to {const_val}"));
        }

        // Using `const` implies no other rules
        return is_valid;
      }

      if self.defined_only && T::try_from(val).is_err() {
        handle_violation!(DefinedOnly, "must be a known enum value".to_string());
      }

      if let Some(allowed_list) = &self.in_
        && !allowed_list.items.contains(&val)
      {
        handle_violation!(
          In,
          format!(
            "must be one of these values: {}",
            i32::format_list(allowed_list)
          )
        );
      }

      if let Some(forbidden_list) = &self.not_in
        && forbidden_list.items.contains(&val)
      {
        handle_violation!(
          NotIn,
          format!(
            "cannot be one of these values: {}",
            i32::format_list(forbidden_list)
          )
        );
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

  fn as_proto_option(&self) -> Option<ProtoOption> {
    Some(self.clone().into())
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
      outer_rules.set("enum", OptionValue::Message(rules.into()));
    }

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
