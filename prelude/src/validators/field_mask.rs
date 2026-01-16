mod builder;
pub use builder::FieldMaskValidatorBuilder;
use proto_types::FieldMask;

use super::*;

#[non_exhaustive]
#[derive(Clone, Debug, Default)]
pub struct FieldMaskValidator {
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  pub cel: Vec<CelProgram>,

  pub ignore: Ignore,

  /// Specifies that the field must be set in order to be valid.
  pub required: bool,

  /// Specifies that only the values in this list will be considered valid for this field.
  pub in_: Option<StaticLookup<SharedStr>>,

  /// Specifies that the values in this list will be considered NOT valid for this field.
  pub not_in: Option<StaticLookup<SharedStr>>,

  /// Specifies that only this specific value will be considered valid for this field.
  pub const_: Option<StaticLookup<SharedStr>>,
}

impl ProtoValidator for FieldMask {
  type Target = Self;
  type Validator = FieldMaskValidator;
  type Builder = FieldMaskValidatorBuilder;
}

impl Validator<FieldMask> for FieldMaskValidator {
  type Target = FieldMask;
  type UniqueStore<'a>
    = LinearRefStore<'a, FieldMask>
  where
    Self: 'a;

  impl_testing_methods!();

  #[inline]
  fn make_unique_store<'a>(&self, cap: usize) -> Self::UniqueStore<'a> {
    LinearRefStore::default_with_capacity(cap)
  }

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

  fn validate(&self, ctx: &mut ValidationCtx, val: Option<&Self::Target>) -> bool {
    handle_ignore_always!(&self.ignore);

    let mut is_valid = true;

    if let Some(val) = val {
      if let Some(const_val) = &self.const_ {
        let const_val_len = const_val.items.len();

        let is_valid = if const_val_len != val.paths.len() {
          false
        } else if const_val_len <= 64 {
          Self::validate_exact_small(&const_val.items, &val.paths)
        } else {
          Self::validate_exact_large(&const_val.items, &val.paths, const_val_len)
        };

        if !is_valid {
          ctx.add_violation(
            FIELD_MASK_CONST_VIOLATION,
            &format!(
              "must contain exactly these paths: [ {} ]",
              val.paths.join(", ")
            ),
          );
        }

        // Using `const` implies no other rules
        return is_valid;
      }

      if let Some(allowed_paths) = &self.in_ {
        for path in &val.paths {
          if !allowed_paths.items.contains(path.as_str()) {
            let err = ["can only contain these paths: ", &allowed_paths.items_str].concat();

            ctx.add_violation(FIELD_MASK_IN_VIOLATION, &err);

            if ctx.fail_fast {
              return false;
            } else {
              is_valid = false;

              break;
            }
          }
        }
      }

      if let Some(forbidden_paths) = &self.not_in {
        for path in &val.paths {
          if forbidden_paths.items.contains(path.as_str()) {
            let err = [
              "cannot contain one of these paths: ",
              &forbidden_paths.items_str,
            ]
            .concat();

            ctx.add_violation(FIELD_MASK_NOT_IN_VIOLATION, &err);

            if ctx.fail_fast {
              return false;
            } else {
              is_valid = false;

              break;
            }
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

        is_valid = cel_ctx.execute_programs();
      }
    } else if self.required {
      ctx.add_required_violation();
      is_valid = false;
    }

    is_valid
  }
}

impl FieldMaskValidator {
  #[inline]
  fn validate_exact_small(const_val: &SortedList<SharedStr>, input_paths: &[String]) -> bool {
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
  #[inline]
  fn validate_exact_large(
    const_val: &SortedList<SharedStr>,
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
  fn from(validator: FieldMaskValidator) -> Self {
    let mut rules = OptionMessageBuilder::new();

    if let Some(const_val) = validator.const_ {
      let mut msg_val = OptionMessageBuilder::new();

      msg_val.set("paths", OptionValue::new_list(const_val.items));

      rules.set("const", OptionValue::Message(msg_val.into()));
    }

    rules
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

    outer_rules.set("field_mask", OptionValue::Message(rules.into()));

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
