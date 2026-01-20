mod builder;
pub use builder::RepeatedValidatorBuilder;

use proto_types::protovalidate::field_path_element::Subscript;

use super::*;

#[non_exhaustive]
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

  pub error_messages: Option<ErrorMessages<RepeatedViolation>>,
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
      error_messages: None,
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
      // If the items are messages, the items validator
      // will be set no matter what
      items: T::default_validator(),
      min_items: None,
      max_items: None,
      unique: false,
      ignore: Ignore::Unspecified,
      error_messages: None,
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
  T::Target: TryIntoCel + Sized + Clone,
{
  type Target = [T::Target];
  type Validator = RepeatedValidator<T>;
  type Builder = RepeatedValidatorBuilder<T>;
}

impl<T, S> ValidatorBuilderFor<Vec<T>> for RepeatedValidatorBuilder<T, S>
where
  S: builder::State,
  T: ProtoValidator,
  T::Target: TryIntoCel + Sized + Clone,
{
  type Target = [T::Target];
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
  T::Target: TryIntoCel + Sized + Clone,
{
  type Target = [T::Target];
  type UniqueStore<'a>
    = UnsupportedStore<Self::Target>
  where
    Self: 'a;

  #[cfg(feature = "cel")]
  fn check_cel_programs(&self) -> Result<(), Vec<CelError>> {
    self.check_cel_programs_with(Vec::new())
  }

  #[inline]
  #[doc(hidden)]
  fn make_unique_store<'a>(&self, _size: usize) -> Self::UniqueStore<'a>
  where
    T: 'a,
  {
    UnsupportedStore::new()
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
  fn check_cel_programs_with(
    &self,
    val: <Self::Target as ToOwned>::Owned,
  ) -> Result<(), Vec<CelError>> {
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

  fn validate_core<V>(&self, ctx: &mut ValidationCtx, val: Option<&V>) -> bool
  where
    V: Borrow<Self::Target> + ?Sized,
  {
    handle_ignore_always!(&self.ignore);
    handle_ignore_if_zero_value!(&self.ignore, val.is_none_or(|v| v.borrow().is_empty()));

    let mut is_valid = true;

    if let Some(val) = val {
      let val = val.borrow();

      macro_rules! handle_violation {
        ($id:ident, $default:expr) => {
          ctx.add_repeated_violation(
            RepeatedViolation::$id,
            self
              .error_messages
              .as_deref()
              .and_then(|map| map.get(&RepeatedViolation::$id))
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

      if let Some(min) = self.min_items
        && val.len() < min
      {
        handle_violation!(
          MinItems,
          format!("must contain at least {min} item{}", pluralize!(min))
        );
      }

      if let Some(max) = self.max_items
        && val.len() > max
      {
        handle_violation!(
          MaxItems,
          format!("cannot contain more than {max} item{}", pluralize!(max))
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

            if !has_unique_values_so_far && ctx.fail_fast {
              break;
            }
          }

          if let Some(validator) = items_validator {
            let _ = ctx
              .field_context
              .as_mut()
              .map(|fc| fc.subscript = Some(Subscript::Index(i as u64)));
            let _ = ctx
              .field_context
              .as_mut()
              .map(|fc| fc.field_kind = FieldKind::RepeatedItem);

            is_valid = validator.validate_core(ctx, Some(value));

            if !is_valid && ctx.fail_fast {
              return false;
            }
          }
        }

        let _ = ctx
          .field_context
          .as_mut()
          .map(|fc| fc.subscript = None);
        let _ = ctx
          .field_context
          .as_mut()
          .map(|fc| fc.field_kind = FieldKind::default());
      }

      if !has_unique_values_so_far {
        handle_violation!(Unique, "must contain unique values".to_string());
      }

      #[cfg(feature = "cel")]
      if !self.cel.is_empty() {
        match try_convert_to_cel(val.to_owned()) {
          Ok(cel_value) => {
            let cel_ctx = ProgramsExecutionCtx {
              programs: &self.cel,
              value: cel_value,
              ctx,
            };

            is_valid = cel_ctx.execute_programs();
          }
          Err(e) => {
            ctx.violations.push(ViolationCtx {
              kind: ViolationKind::Cel,
              data: e.into_violation(ctx.field_context.as_ref(), &ctx.parent_elements),
            });
            is_valid = false;
          }
        };
      }
    }

    is_valid
  }

  fn into_proto_option(self) -> Option<ProtoOption> {
    Some(self.into())
  }
}

impl<T> From<RepeatedValidator<T>> for ProtoOption
where
  T: ProtoValidator,
{
  fn from(validator: RepeatedValidator<T>) -> Self {
    let mut rules = OptionMessageBuilder::new();

    rules
      .set_boolean("unique", validator.unique)
      .maybe_set("min_items", validator.min_items)
      .maybe_set("max_items", validator.max_items);

    if let Some(items_option) = validator
      .items
      .and_then(|i| i.into_proto_option())
    {
      rules.set("items", items_option.value);
    }

    let mut outer_rules = OptionMessageBuilder::new();

    outer_rules.set("repeated", OptionValue::Message(rules.into()));

    outer_rules
      .add_cel_options(validator.cel)
      .set_ignore(validator.ignore);

    Self {
      name: "(buf.validate.field)".into(),
      value: OptionValue::Message(outer_rules.into()),
    }
  }
}
