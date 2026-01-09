mod builder;
pub use builder::RepeatedValidatorBuilder;

use proto_types::protovalidate::field_path_element::Subscript;

use super::*;

#[derive(Debug)]
pub struct RepeatedValidator<T>
where
  T: ProtoValidator,
{
  _inner_type: PhantomData<T>,

  pub cel: Vec<CelProgram>,
  pub items: Option<T::Validator>,
  /// The minimum amount of items that this field must contain in order to be valid.
  pub min_items: Option<usize>,
  /// The maximum amount of items that this field must contain in order to be valid.
  pub max_items: Option<usize>,
  /// Specifies that this field must contain only unique values (only applies to scalar fields).
  pub unique: bool,
  pub ignore: Ignore,
}

impl<T> Clone for RepeatedValidator<T>
where
  T: ProtoValidator,
{
  #[inline]
  fn clone(&self) -> Self {
    Self {
      _inner_type: PhantomData,
      cel: self.cel.clone(),
      items: self.items.clone(),
      min_items: self.min_items,
      max_items: self.max_items,
      unique: self.unique,
      ignore: self.ignore,
    }
  }
}

impl<T> Default for RepeatedValidator<T>
where
  T: ProtoValidator,
{
  #[inline]
  fn default() -> Self {
    Self {
      _inner_type: PhantomData,
      cel: vec![],
      items: T::default_validator(),
      min_items: None,
      max_items: None,
      unique: false,
      ignore: Ignore::Unspecified,
    }
  }
}

impl<T: AsProtoType> AsProtoField for Vec<T> {
  fn as_proto_field() -> FieldType {
    FieldType::Repeated(T::proto_type())
  }
}

impl<T> ProtoValidator for Vec<T>
where
  T: ProtoValidator,
  T::Target: TryIntoCel,
{
  type Target = Vec<T::Target>;
  type Validator = RepeatedValidator<T>;
  type Builder = RepeatedValidatorBuilder<T>;
}

impl<T, S> ValidatorBuilderFor<Vec<T>> for RepeatedValidatorBuilder<T, S>
where
  S: builder::State,
  T: ProtoValidator,
  T::Target: TryIntoCel,
{
  type Target = Vec<T::Target>;
  type Validator = RepeatedValidator<T>;

  #[inline]
  #[doc(hidden)]
  fn build_validator(self) -> Self::Validator {
    self.build()
  }
}

#[cfg(feature = "cel")]
fn try_convert_to_cel<T: TryIntoCel>(list: Vec<T>) -> Result<::cel::Value, CelError> {
  let values: Vec<::cel::Value> = list
    .into_iter()
    .map(|i| i.try_into_cel())
    .collect::<Result<Vec<::cel::Value>, CelError>>()?;

  Ok(values.into())
}

impl<T> Validator<Vec<T>> for RepeatedValidator<T>
where
  T: ProtoValidator,
  T::Target: TryIntoCel,
{
  type Target = Vec<T::Target>;
  type UniqueStore<'a>
    = UnsupportedStore<Self::Target>
  where
    Self: 'a;

  #[inline]
  #[doc(hidden)]
  fn make_unique_store<'a>(&self, _size: usize) -> Self::UniqueStore<'a>
  where
    T: 'a,
  {
    UnsupportedStore::default()
  }

  fn check_consistency(&self) -> Result<(), Vec<ConsistencyError>> {
    let mut errors = Vec::new();

    #[cfg(feature = "cel")]
    if let Err(e) = self.check_cel_programs() {
      errors.extend(e.into_iter().map(ConsistencyError::from));
    }

    if let Err(e) = check_length_rules(
      None,
      length_rule_value!("min_items", self.min_items),
      length_rule_value!("max_items", self.max_items),
    ) {
      errors.push(e);
    }

    if let Some(items_validator) = &self.items
      && let Err(e) = items_validator.check_consistency()
    {
      errors.extend(e);
    }

    if errors.is_empty() {
      Ok(())
    } else {
      Err(errors)
    }
  }

  #[doc(hidden)]
  fn cel_rules(&self) -> Vec<CelRule> {
    let mut rules: Vec<CelRule> = self.cel.iter().map(|p| p.rule.clone()).collect();

    rules.extend(self.items.iter().flat_map(|i| i.cel_rules()));

    rules
  }

  #[cfg(feature = "cel")]
  fn check_cel_programs_with(&self, val: Self::Target) -> Result<(), Vec<CelError>> {
    let mut errors = Vec::new();

    if !self.cel.is_empty() {
      match try_convert_to_cel(val) {
        Ok(val) => {
          if let Err(e) = test_programs(&self.cel, val) {
            errors.extend(e)
          }
        }
        Err(e) => errors.push(e),
      }
    }

    if let Some(items_validator) = &self.items
      && let Err(e) = items_validator.check_cel_programs()
    {
      errors.extend(e)
    }

    if errors.is_empty() {
      Ok(())
    } else {
      Err(errors)
    }
  }

  fn validate(&self, ctx: &mut ValidationCtx, val: Option<&Self::Target>) {
    handle_ignore_always!(&self.ignore);
    handle_ignore_if_zero_value!(&self.ignore, val.is_none_or(|v| v.is_empty()));

    if let Some(val) = val {
      #[cfg(feature = "cel")]
      if !self.cel.is_empty() {
        match try_convert_to_cel(val.clone()) {
          Ok(cel_value) => {
            let ctx = ProgramsExecutionCtx {
              programs: &self.cel,
              value: cel_value,
              violations: ctx.violations,
              field_context: Some(&ctx.field_context),
              parent_elements: ctx.parent_elements,
            };

            ctx.execute_programs();
          }
          Err(e) => ctx
            .violations
            .push(e.into_violation(Some(&ctx.field_context), ctx.parent_elements)),
        };
      }

      if let Some(min) = &self.min_items
        && val.len() < *min
      {
        ctx.add_violation(
          &REPEATED_MIN_ITEMS_VIOLATION,
          &format!("must contain at least {min} items"),
        );
      }

      if let Some(max) = &self.max_items
        && val.len() > *max
      {
        ctx.add_violation(
          &REPEATED_MAX_ITEMS_VIOLATION,
          &format!("cannot contain more than {max} items"),
        );
      }

      let items_validator = self.items.as_ref();

      // We only create this if there is a `unique` restriction
      let mut unique_store = if self.unique {
        let size = val.len();

        let store = match &self.items {
          Some(v) => v.make_unique_store(size),
          None => {
            <<T as ProtoValidator>::Validator as Validator<T>>::UniqueStore::default_with_capacity(
              size,
            )
          }
        };

        Some(store)
      } else {
        None
      };

      let mut has_unique_values_so_far = true;

      if self.unique || items_validator.is_some() {
        for (i, value) in val.iter().enumerate() {
          if let Some(unique_store) = unique_store.as_mut()
            && has_unique_values_so_far
          {
            has_unique_values_so_far = unique_store.insert(value);
          }

          if let Some(validator) = items_validator {
            ctx.field_context.subscript = Some(Subscript::Index(i as u64));
            ctx.field_context.field_kind = FieldKind::RepeatedItem;

            validator.validate(ctx, Some(value));
          }
        }

        ctx.field_context.subscript = None;
        ctx.field_context.field_kind = FieldKind::Repeated;
      }

      if !has_unique_values_so_far {
        ctx.add_violation(&REPEATED_UNIQUE_VIOLATION, "must contain unique values");
      }
    }
  }
}

impl<T> From<RepeatedValidator<T>> for ProtoOption
where
  T: ProtoValidator,
{
  fn from(validator: RepeatedValidator<T>) -> Self {
    let mut rules = OptionMessageBuilder::new();

    rules
      .set_boolean(&UNIQUE, validator.unique)
      .maybe_set(&MIN_ITEMS, validator.min_items)
      .maybe_set(&MAX_ITEMS, validator.max_items);

    if let Some(items_option) = validator.items {
      let items_schema: Self = items_option.into();

      rules.set(ITEMS.clone(), items_schema.value);
    }

    let mut outer_rules = OptionMessageBuilder::new();

    outer_rules.set(REPEATED.clone(), OptionValue::Message(rules.into()));

    outer_rules
      .add_cel_options(validator.cel)
      .set_ignore(validator.ignore);

    Self {
      name: BUF_VALIDATE_FIELD.clone(),
      value: OptionValue::Message(outer_rules.into()),
    }
  }
}
