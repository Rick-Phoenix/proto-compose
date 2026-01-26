mod builder;
pub use builder::DurationValidatorBuilder;

use proto_types::Duration;

use super::*;

#[non_exhaustive]
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DurationValidator {
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  pub cel: Vec<CelProgram>,

  pub ignore: Ignore,

  /// Specifies that the field must be set in order to be valid.
  pub required: bool,

  /// Specifies that only the values in this list will be considered valid for this field.
  pub in_: Option<SortedList<Duration>>,

  /// Specifies that the values in this list will be considered NOT valid for this field.
  pub not_in: Option<SortedList<Duration>>,

  /// Specifies that only this specific value will be considered valid for this field.
  pub const_: Option<Duration>,

  /// Specifies that the value must be smaller than the indicated amount in order to pass validation.
  pub lt: Option<Duration>,

  /// Specifies that the value must be equal to or smaller than the indicated amount in order to pass validation.
  pub lte: Option<Duration>,

  /// Specifies that the value must be greater than the indicated amount in order to pass validation.
  pub gt: Option<Duration>,

  /// Specifies that the value must be equal to or greater than the indicated amount in order to pass validation.
  pub gte: Option<Duration>,

  pub error_messages: Option<ErrorMessages<DurationViolation>>,
}

impl Validator<Duration> for DurationValidator {
  type Target = Duration;

  impl_testing_methods!();

  #[inline(never)]
  #[cold]
  fn check_consistency(&self) -> Result<(), Vec<ConsistencyError>> {
    let mut errors = Vec::new();

    macro_rules! check_prop_some {
      ($($id:ident),*) => {
        $(self.$id.is_some()) ||*
      };
    }

    if self.const_.is_some()
      && (!self.cel.is_empty() || check_prop_some!(in_, not_in, lt, lte, gt, gte))
    {
      errors.push(ConsistencyError::ConstWithOtherRules);
    }

    if let Some(custom_messages) = self.error_messages.as_deref() {
      let mut unused_messages: Vec<String> = Vec::new();

      for key in custom_messages.keys() {
        macro_rules! check_unused_messages {
          ($($name:ident),*) => {
            paste! {
              match key {
                DurationViolation::Required => self.required,
                DurationViolation::In => self.in_.is_some(),
                DurationViolation::Const => self.const_.is_some(),
                $(DurationViolation::[< $name:camel >] => self.$name.is_some(),)*
                _ => true,
              }
            }
          };
        }

        let is_used = check_unused_messages!(gt, gte, lt, lte, not_in);

        if !is_used {
          unused_messages.push(format!("{key:?}"));
        }
      }

      if !unused_messages.is_empty() {
        errors.push(ConsistencyError::UnusedCustomMessages(unused_messages));
      }
    }

    #[cfg(feature = "cel")]
    if let Err(e) = self.check_cel_programs() {
      errors.extend(e.into_iter().map(ConsistencyError::from));
    }

    if let Err(e) = check_list_rules(self.in_.as_ref(), self.not_in.as_ref()) {
      errors.push(e.into());
    }

    if let Err(e) = check_comparable_rules(self.lt, self.lte, self.gt, self.gte) {
      errors.push(e);
    }

    if errors.is_empty() {
      Ok(())
    } else {
      Err(errors)
    }
  }

  fn validate_core<V>(&self, ctx: &mut ValidationCtx, val: Option<&V>) -> ValidationResult
  where
    V: Borrow<Self::Target> + ?Sized,
  {
    handle_ignore_always!(&self.ignore);

    let mut is_valid = IsValid::Yes;

    macro_rules! handle_violation {
      ($id:ident, $default:expr) => {
        is_valid &= ctx.add_violation(
          ViolationKind::Duration(DurationViolation::$id),
          self
            .error_messages
            .as_deref()
            .and_then(|map| map.get(&DurationViolation::$id))
            .map(|m| Cow::Borrowed(m.as_ref()))
            .unwrap_or_else(|| Cow::Owned($default)),
        )?;
      };
    }

    if let Some(val) = val {
      let val = *val.borrow();

      if let Some(const_val) = self.const_ {
        if val != const_val {
          handle_violation!(
            Const,
            format!("must be equal to {}", const_val.to_human_readable_string())
          );
        }

        // Using `const` implies no other rules
        return Ok(is_valid);
      }

      if let Some(gt) = self.gt
        && val <= gt
      {
        handle_violation!(
          Gt,
          format!("must be longer than {}", gt.to_human_readable_string())
        );
      }

      if let Some(gte) = self.gte
        && val < gte
      {
        handle_violation!(
          Gte,
          format!(
            "must be longer than or equal to {}",
            gte.to_human_readable_string()
          )
        );
      }

      if let Some(lt) = self.lt
        && val >= lt
      {
        handle_violation!(
          Lt,
          format!("must be shorter than {}", lt.to_human_readable_string())
        );
      }

      if let Some(lte) = self.lte
        && val > lte
      {
        handle_violation!(
          Lte,
          format!(
            "must be shorter than or equal to {}",
            lte.to_human_readable_string()
          )
        );
      }

      if let Some(allowed_list) = &self.in_
        && !allowed_list.contains(&val)
      {
        handle_violation!(
          In,
          format!(
            "must be one of these values: {}",
            Duration::format_list(allowed_list)
          )
        );
      }

      if let Some(forbidden_list) = &self.not_in
        && forbidden_list.items.contains(&val)
      {
        handle_violation!(
          NotIn,
          format!(
            "must be one of these values: {}",
            Duration::format_list(forbidden_list)
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

        is_valid &= cel_ctx.execute_programs()?;
      }
    } else if self.required {
      handle_violation!(Required, "is required".to_string());
    }

    Ok(is_valid)
  }

  #[inline(never)]
  #[cold]
  fn schema(&self) -> Option<ValidatorSchema> {
    Some(ValidatorSchema {
      schema: self.clone().into(),
      cel_rules: self.cel_rules(),
      imports: vec!["buf/validate/validate.proto".into()],
    })
  }
}

impl From<DurationValidator> for ProtoOption {
  #[inline(never)]
  #[cold]
  fn from(validator: DurationValidator) -> Self {
    let mut rules = OptionMessageBuilder::new();

    rules
      .maybe_set("const", validator.const_)
      .maybe_set("lt", validator.lt)
      .maybe_set("lte", validator.lte)
      .maybe_set("gt", validator.gt)
      .maybe_set("gte", validator.gte)
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
      outer_rules.set("duration", OptionValue::Message(rules.build()));
    }

    outer_rules
      .add_cel_options(validator.cel)
      .set_required(validator.required)
      .set_ignore(validator.ignore);

    Self {
      name: "(buf.validate.field)".into(),
      value: OptionValue::Message(outer_rules.build()),
    }
  }
}
