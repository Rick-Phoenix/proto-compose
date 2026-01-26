mod builder;
pub use builder::FieldMaskValidatorBuilder;
use proto_types::FieldMask;

use super::*;

#[non_exhaustive]
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FieldMaskValidator {
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  pub cel: Vec<CelProgram>,

  pub ignore: Ignore,

  /// Specifies that the field must be set in order to be valid.
  pub required: bool,

  /// Specifies that only the values in this list will be considered valid for this field.
  pub in_: Option<SortedList<FixedStr>>,

  /// Specifies that the values in this list will be considered NOT valid for this field.
  pub not_in: Option<SortedList<FixedStr>>,

  /// Specifies that only this specific value will be considered valid for this field.
  pub const_: Option<SortedList<FixedStr>>,

  pub error_messages: Option<ErrorMessages<FieldMaskViolation>>,
}

impl ProtoValidation for FieldMask {
  type Target = Self;
  type Stored = Self;
  type Validator = FieldMaskValidator;
  type Builder = FieldMaskValidatorBuilder;

  type UniqueStore<'a>
    = LinearRefStore<'a, Self>
  where
    Self: 'a;

  #[inline]
  fn make_unique_store<'a>(_: &Self::Validator, cap: usize) -> Self::UniqueStore<'a> {
    LinearRefStore::default_with_capacity(cap)
  }
}

impl Validator<FieldMask> for FieldMaskValidator {
  type Target = FieldMask;

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

    if self.const_.is_some() && (!self.cel.is_empty() || check_prop_some!(in_, not_in)) {
      errors.push(ConsistencyError::ConstWithOtherRules);
    }

    if let Some(custom_messages) = self.error_messages.as_deref() {
      let mut unused_messages: Vec<String> = Vec::new();

      for key in custom_messages.keys() {
        let is_used = match key {
          FieldMaskViolation::Required => self.required,
          FieldMaskViolation::In => self.in_.is_some(),
          FieldMaskViolation::Const => self.const_.is_some(),
          FieldMaskViolation::NotIn => self.not_in.is_some(),
          _ => true,
        };

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
          ViolationKind::FieldMask(FieldMaskViolation::$id),
          self
            .error_messages
            .as_deref()
            .and_then(|map| map.get(&FieldMaskViolation::$id))
            .map(|m| Cow::Borrowed(m.as_ref()))
            .unwrap_or_else(|| Cow::Owned($default)),
        )?;
      };
    }

    if let Some(val) = val {
      let val = val.borrow();

      if let Some(const_val) = &self.const_ {
        let const_val_len = const_val.items.len();

        let matches_const = if const_val_len != val.paths.len() {
          false
        } else if const_val_len <= 64 {
          Self::validate_exact_small(const_val, &val.paths)
        } else {
          Self::validate_exact_large(const_val, &val.paths, const_val_len)
        };

        if !matches_const {
          handle_violation!(
            Const,
            format!(
              "must contain exactly these paths: [ {} ]",
              val.paths.join(", ")
            )
          );
        }

        // Using `const` implies no other rules
        return Ok(is_valid);
      }

      if let Some(allowed_paths) = &self.in_ {
        for path in &val.paths {
          if !allowed_paths.contains(path.as_str()) {
            handle_violation!(
              In,
              format!(
                "can only contain these paths: {}",
                FixedStr::format_list(allowed_paths)
              )
            );

            break;
          }
        }
      }

      if let Some(forbidden_paths) = &self.not_in {
        for path in &val.paths {
          if forbidden_paths.contains(path.as_str()) {
            handle_violation!(
              NotIn,
              format!(
                "cannot contain one of these paths: {}",
                FixedStr::format_list(forbidden_paths)
              )
            );

            break;
          }
        }
      }

      #[cfg(feature = "cel")]
      if !self.cel.is_empty() {
        let cel_ctx = ProgramsExecutionCtx {
          programs: &self.cel,
          value: val.clone(),
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

impl FieldMaskValidator {
  fn validate_exact_small(const_val: &SortedList<FixedStr>, input_paths: &[String]) -> bool {
    let mut visited_mask: u64 = 0;

    for path in input_paths {
      match const_val.binary_search_by(|probe| probe.as_str().cmp(path)) {
        Ok(idx) => {
          let bit = 1 << idx;
          // Check if bit is already 1 (Duplicate input)
          if (visited_mask & bit) != 0 {
            return false;
          }
          // Set bit to 1
          visited_mask |= bit;
        }
        Err(_) => return false,
      }
    }
    true
  }

  // Fallback: One allocation, Heap-based checklist
  // Only used in the rare case that a FieldMask has more than 64 paths in it
  fn validate_exact_large(
    const_val: &SortedList<FixedStr>,
    input_paths: &[String],
    len: usize,
  ) -> bool {
    // Create a checklist of size N, initialized to false
    let mut visited = vec![false; len];

    for path in input_paths {
      match const_val.binary_search_by(|probe| probe.as_str().cmp(path)) {
        Ok(idx) => {
          if visited[idx] {
            return false;
          }
          visited[idx] = true;
        }
        Err(_) => return false,
      }
    }
    true
  }
}

impl From<FieldMaskValidator> for ProtoOption {
  #[inline(never)]
  #[cold]
  fn from(validator: FieldMaskValidator) -> Self {
    let mut rules = OptionMessageBuilder::new();

    if let Some(const_val) = validator.const_ {
      let mut msg_val = OptionMessageBuilder::new();

      msg_val.set("paths", OptionValue::new_list(const_val));

      rules.set("const", OptionValue::Message(msg_val.into()));
    }

    rules
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
      outer_rules.set("field_mask", OptionValue::Message(rules.into()));
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
