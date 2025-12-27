pub mod builder;
pub use builder::FieldMaskValidatorBuilder;
use builder::state::State;
use proto_types::FieldMask;

use super::*;

#[derive(Clone, Debug)]
pub struct FieldMaskValidator {
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  pub cel: Vec<&'static CelProgram>,

  pub ignore: Ignore,

  /// Specifies that the field must be set in order to be valid.
  pub required: bool,

  /// Specifies that only the values in this list will be considered valid for this field.
  pub in_: Option<&'static StaticLookup<&'static str>>,

  /// Specifies that the values in this list will be considered NOT valid for this field.
  pub not_in: Option<&'static StaticLookup<&'static str>>,

  /// Specifies that only this specific value will be considered valid for this field.
  pub const_: Option<&'static StaticLookup<&'static str>>,
}

impl<S: State> ValidatorBuilderFor<FieldMask> for FieldMaskValidatorBuilder<S> {
  type Target = FieldMask;
  type Validator = FieldMaskValidator;

  fn build_validator(self) -> Self::Validator {
    self.build()
  }
}

impl Validator<FieldMask> for FieldMaskValidator {
  type Target = FieldMask;
  type UniqueStore<'a>
    = LinearRefStore<'a, FieldMask>
  where
    Self: 'a;

  impl_testing_methods!();

  fn make_unique_store<'a>(&self, cap: usize) -> Self::UniqueStore<'a> {
    LinearRefStore::default_with_capacity(cap)
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

    let mut violations_agg = Violations::new();
    let violations = &mut violations_agg;

    if let Some(val) = val {
      if let Some(const_val) = self.const_ {
        let const_val_len = const_val.items.len();

        let is_valid = if const_val_len != val.paths.len() {
          false
        } else if const_val_len <= 64 {
          Self::validate_exact_small(&const_val.items, &val.paths)
        } else {
          Self::validate_exact_large(&const_val.items, &val.paths, const_val_len)
        };

        if !is_valid {
          violations.add(
            field_context,
            parent_elements,
            &FIELD_MASK_CONST_VIOLATION,
            &format!(
              "must contain exactly these paths: [ {} ]",
              val.paths.join(", ")
            ),
          );
        }
      }

      if let Some(allowed_paths) = self.in_ {
        for path in &val.paths {
          if !allowed_paths.items.contains(&path.as_str()) {
            let err = ["can only contain these paths: ", &allowed_paths.items_str].concat();

            violations.add(
              field_context,
              parent_elements,
              &FIELD_MASK_IN_VIOLATION,
              &err,
            );

            break;
          }
        }
      }

      if let Some(forbidden_paths) = self.not_in {
        for path in &val.paths {
          if forbidden_paths.items.contains(&path.as_str()) {
            let err = [
              "cannot contain one of these paths: ",
              &forbidden_paths.items_str,
            ]
            .concat();

            violations.add(
              field_context,
              parent_elements,
              &FIELD_MASK_NOT_IN_VIOLATION,
              &err,
            );

            break;
          }
        }
      }

      #[cfg(feature = "cel")]
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

impl FieldMaskValidator {
  #[inline]
  fn validate_exact_small(
    const_val: &'static SortedList<&'static str>,
    input_paths: &[String],
  ) -> bool {
    let mut visited_mask: u64 = 0;

    for path in input_paths {
      match const_val.binary_search(&path.as_str()) {
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
  #[inline]
  fn validate_exact_large(
    const_val: &'static SortedList<&'static str>,
    input_paths: &[String],
    len: usize,
  ) -> bool {
    // Create a checklist of size N, initialized to false
    let mut visited = vec![false; len];

    for path in input_paths {
      match const_val.binary_search(&path.as_str()) {
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
  fn from(validator: FieldMaskValidator) -> Self {
    let mut rules: OptionValueList = Vec::new();

    if let Some(const_val) = validator.const_ {
      let mut msg_val: OptionValueList = Vec::new();

      msg_val.push((PATHS.clone(), OptionValue::new_list(const_val.items.iter())));

      rules.push((CONST_.clone(), OptionValue::Message(msg_val.into())));
    }

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

    outer_rules.push((FIELD_MASK.clone(), OptionValue::Message(rules.into())));

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
