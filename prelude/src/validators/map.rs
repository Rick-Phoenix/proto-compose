mod builder;
pub use builder::MapValidatorBuilder;

use proto_types::protovalidate::{
  field_path_element::Subscript, violations_data::map_violations::*,
};

use super::*;

#[non_exhaustive]
#[derive(Debug)]
pub struct MapValidator<K, V>
where
  K: ProtoValidator,
  V: ProtoValidator,
{
  /// The validation rules to apply to the keys of this map field.
  pub keys: Option<K::Validator>,

  _key_type: PhantomData<K>,
  _value_type: PhantomData<V>,

  pub cel: Vec<CelProgram>,

  /// The validation rules to apply to the keys of this map field.
  pub values: Option<V::Validator>,
  /// The minimum amount of key-value pairs that this field should have in order to be valid.
  pub min_pairs: Option<usize>,
  /// The maximum amount of key-value pairs that this field should have in order to be valid.
  pub max_pairs: Option<usize>,
  pub ignore: Ignore,
}

impl<K: AsProtoMapKey, V: AsProtoType> AsProtoField for HashMap<K, V> {
  fn as_proto_field() -> FieldType {
    FieldType::Map {
      keys: K::as_proto_map_key(),
      values: V::proto_type(),
    }
  }
}

impl<K: AsProtoMapKey, V: AsProtoType> AsProtoField for BTreeMap<K, V> {
  fn as_proto_field() -> FieldType {
    FieldType::Map {
      keys: K::as_proto_map_key(),
      values: V::proto_type(),
    }
  }
}

impl<K, V> ProtoValidator for HashMap<K, V>
where
  K: ProtoValidator,
  V: ProtoValidator,
  K::Target: Clone + Into<Subscript> + Default + Eq + Hash + IntoCelKey,
  V::Target: Default + TryIntoCel + Clone,
{
  type Target = HashMap<K::Target, V::Target>;

  type Validator = MapValidator<K, V>;
  type Builder = MapValidatorBuilder<K, V>;
}

impl<K, V> ProtoValidator for BTreeMap<K, V>
where
  K: ProtoValidator,
  V: ProtoValidator,
  K::Target: Clone + Into<Subscript> + Sized,
  V::Target: Sized + Clone,
{
  type Target = BTreeMap<K::Target, V::Target>;

  type Validator = MapValidator<K, V>;
  type Builder = MapValidatorBuilder<K, V>;
}

impl<K, V, S: builder::state::State> ValidatorBuilderFor<BTreeMap<K, V>>
  for MapValidatorBuilder<K, V, S>
where
  K: ProtoValidator,
  V: ProtoValidator,
  K::Target: Clone + Into<Subscript> + Sized,
  V::Target: Sized + Clone,
{
  type Target = BTreeMap<K::Target, V::Target>;
  type Validator = MapValidator<K, V>;

  #[inline]
  #[doc(hidden)]
  fn build_validator(self) -> Self::Validator {
    self.build()
  }
}

impl<K, V, S: builder::state::State> ValidatorBuilderFor<HashMap<K, V>>
  for MapValidatorBuilder<K, V, S>
where
  K: ProtoValidator,
  V: ProtoValidator,
  K::Target: Clone + Into<Subscript> + Default + Eq + Hash + IntoCelKey,
  V::Target: Default + TryIntoCel + Clone,
{
  type Target = HashMap<K::Target, V::Target>;
  type Validator = MapValidator<K, V>;

  #[inline]
  #[doc(hidden)]
  fn build_validator(self) -> Self::Validator {
    self.build()
  }
}

impl<K, V> Default for MapValidator<K, V>
where
  K: ProtoValidator,
  V: ProtoValidator,
{
  #[inline]
  fn default() -> Self {
    Self {
      keys: None,
      _key_type: PhantomData,
      _value_type: PhantomData,
      cel: vec![],
      values: V::default_validator(),
      min_pairs: None,
      max_pairs: None,
      ignore: Ignore::Unspecified,
    }
  }
}

impl<K, V> Clone for MapValidator<K, V>
where
  K: ProtoValidator,
  V: ProtoValidator,
{
  #[inline]
  fn clone(&self) -> Self {
    Self {
      keys: self.keys.clone(),
      _key_type: PhantomData,
      _value_type: PhantomData,
      cel: self.cel.clone(),
      values: self.values.clone(),
      min_pairs: self.min_pairs,
      max_pairs: self.max_pairs,
      ignore: self.ignore,
    }
  }
}

#[cfg(feature = "cel")]
fn try_convert_to_cel<K, V>(map: HashMap<K, V>) -> Result<::cel::Value, CelError>
where
  K: IntoCelKey,
  V: TryIntoCel,
{
  let mut cel_map: HashMap<::cel::objects::Key, ::cel::Value> = HashMap::new();

  for (k, v) in map {
    cel_map.insert(k.into(), v.try_into_cel()?);
  }

  Ok(::cel::Value::Map(::cel::objects::Map {
    map: Arc::new(cel_map),
  }))
}

impl<K, V> Validator<BTreeMap<K, V>> for MapValidator<K, V>
where
  K: ProtoValidator,
  V: ProtoValidator,
  K::Target: Clone + Into<Subscript> + Sized,
  V::Target: Sized + Clone,
{
  type Target = BTreeMap<K::Target, V::Target>;
  type UniqueStore<'a>
    = UnsupportedStore<Self::Target>
  where
    Self: 'a;

  #[cfg(feature = "cel")]
  fn check_cel_programs(&self) -> Result<(), Vec<CelError>> {
    self.check_cel_programs_with(BTreeMap::default())
  }

  #[inline]
  #[doc(hidden)]
  fn make_unique_store<'a>(&self, _: usize) -> Self::UniqueStore<'a>
  where
    Self: 'a,
  {
    UnsupportedStore::new()
  }

  fn check_consistency(&self) -> Result<(), Vec<ConsistencyError>> {
    let mut errors = Vec::new();

    if let Err(e) = check_length_rules(
      None,
      length_rule_value!("min_pairs", self.min_pairs),
      length_rule_value!("max_pairs", self.max_pairs),
    ) {
      errors.push(e);
    }

    if let Some(keys_validator) = &self.keys
      && let Err(e) = keys_validator.check_consistency()
    {
      errors.extend(e);
    }

    if let Some(values_validator) = &self.values
      && let Err(e) = values_validator.check_consistency()
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
    vec![]
  }

  #[cfg(feature = "cel")]
  fn check_cel_programs_with(&self, _val: Self::Target) -> Result<(), Vec<CelError>> {
    Ok(())
  }

  fn validate(&self, ctx: &mut ValidationCtx, val: Option<&Self::Target>) -> bool {
    handle_ignore_always!(&self.ignore);
    handle_ignore_if_zero_value!(&self.ignore, val.is_none_or(|v| v.is_empty()));

    let mut is_valid = true;

    if let Some(val) = val {
      if let Some(min_pairs) = self.min_pairs
        && val.len() < min_pairs
      {
        ctx.add_violation(
          MAP_MIN_PAIRS_VIOLATION,
          &format!("must contain at least {min_pairs} pairs"),
        );
        handle_violation!(is_valid, ctx);
      }

      if let Some(max_pairs) = self.max_pairs
        && val.len() > max_pairs
      {
        ctx.add_violation(
          MAP_MAX_PAIRS_VIOLATION,
          &format!("cannot contain more than {max_pairs} pairs"),
        );
        handle_violation!(is_valid, ctx);
      }

      let keys_validator = self.keys.as_ref();

      let values_validator = self.values.as_ref();

      if keys_validator.is_some() || values_validator.is_some() {
        for (k, v) in val {
          let _ = ctx
            .field_context
            .as_mut()
            .map(|fc| fc.subscript = Some(k.clone().into()));

          if let Some(validator) = keys_validator {
            let _ = ctx
              .field_context
              .as_mut()
              .map(|fc| fc.field_kind = FieldKind::MapKey);

            is_valid = validator.validate(ctx, Some(k));

            if !is_valid && ctx.fail_fast {
              return false;
            }
          }

          if let Some(validator) = values_validator {
            let _ = ctx
              .field_context
              .as_mut()
              .map(|fc| fc.field_kind = FieldKind::MapValue);

            is_valid = validator.validate(ctx, Some(v));

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
    }

    is_valid
  }
}

impl<K, V> Validator<HashMap<K, V>> for MapValidator<K, V>
where
  K: ProtoValidator,
  V: ProtoValidator,
  K::Target: Clone + Into<Subscript> + Default + Eq + Hash + IntoCelKey,
  V::Target: Default + TryIntoCel + Clone,
{
  type Target = HashMap<K::Target, V::Target>;
  type UniqueStore<'a>
    = UnsupportedStore<Self::Target>
  where
    Self: 'a;

  #[cfg(feature = "cel")]
  fn check_cel_programs(&self) -> Result<(), Vec<CelError>> {
    <Self as Validator<HashMap<K, V>>>::check_cel_programs_with(self, HashMap::default())
  }

  #[inline]
  #[doc(hidden)]
  fn make_unique_store<'a>(&self, _: usize) -> Self::UniqueStore<'a>
  where
    Self: 'a,
  {
    UnsupportedStore::new()
  }

  fn check_consistency(&self) -> Result<(), Vec<ConsistencyError>> {
    let mut errors = Vec::new();

    #[cfg(feature = "cel")]
    if let Err(e) = <Self as Validator<HashMap<K, V>>>::check_cel_programs(self) {
      errors.extend(e.into_iter().map(ConsistencyError::from));
    }

    if let Err(e) = check_length_rules(
      None,
      length_rule_value!("min_pairs", self.min_pairs),
      length_rule_value!("max_pairs", self.max_pairs),
    ) {
      errors.push(e);
    }

    if let Some(keys_validator) = &self.keys
      && let Err(e) = keys_validator.check_consistency()
    {
      errors.extend(e);
    }

    if let Some(values_validator) = &self.values
      && let Err(e) = values_validator.check_consistency()
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

    rules.extend(self.keys.iter().flat_map(|k| k.cel_rules()));
    rules.extend(self.values.iter().flat_map(|v| v.cel_rules()));

    rules
  }

  #[cfg(feature = "cel")]
  fn check_cel_programs_with(&self, val: Self::Target) -> Result<(), Vec<CelError>> {
    let mut errors: Vec<CelError> = Vec::new();

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

    if let Some(key_validator) = &self.keys {
      match key_validator.check_cel_programs() {
        Ok(()) => {}
        Err(errs) => errors.extend(errs),
      };
    }

    if let Some(values_validator) = &self.values {
      match values_validator.check_cel_programs() {
        Ok(()) => {}
        Err(errs) => errors.extend(errs),
      };
    }

    if errors.is_empty() {
      Ok(())
    } else {
      Err(errors)
    }
  }

  fn validate(&self, ctx: &mut ValidationCtx, val: Option<&Self::Target>) -> bool {
    handle_ignore_always!(&self.ignore);
    handle_ignore_if_zero_value!(&self.ignore, val.is_none_or(|v| v.is_empty()));

    let mut is_valid = true;

    if let Some(val) = val {
      if let Some(min_pairs) = self.min_pairs
        && val.len() < min_pairs
      {
        ctx.add_violation(
          MAP_MIN_PAIRS_VIOLATION,
          &format!("must contain at least {min_pairs} pairs"),
        );
        handle_violation!(is_valid, ctx);
      }

      if let Some(max_pairs) = self.max_pairs
        && val.len() > max_pairs
      {
        ctx.add_violation(
          MAP_MAX_PAIRS_VIOLATION,
          &format!("cannot contain more than {max_pairs} pairs"),
        );
        handle_violation!(is_valid, ctx);
      }

      let keys_validator = self.keys.as_ref();

      let values_validator = self.values.as_ref();

      if keys_validator.is_some() || values_validator.is_some() {
        for (k, v) in val {
          let _ = ctx
            .field_context
            .as_mut()
            .map(|fc| fc.subscript = Some(k.clone().into()));

          if let Some(validator) = keys_validator {
            let _ = ctx
              .field_context
              .as_mut()
              .map(|fc| fc.field_kind = FieldKind::MapKey);

            is_valid = validator.validate(ctx, Some(k));

            if !is_valid && ctx.fail_fast {
              return false;
            }
          }

          if let Some(validator) = values_validator {
            let _ = ctx
              .field_context
              .as_mut()
              .map(|fc| fc.field_kind = FieldKind::MapValue);

            is_valid = validator.validate(ctx, Some(v));

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

      #[cfg(feature = "cel")]
      if !self.cel.is_empty() {
        match try_convert_to_cel(val.clone()) {
          Ok(cel_value) => {
            let cel_ctx = ProgramsExecutionCtx {
              programs: &self.cel,
              value: cel_value,
              ctx,
            };

            is_valid = cel_ctx.execute_programs();
          }
          Err(e) => {
            ctx
              .violations
              .push(e.into_violation(ctx.field_context.as_ref(), &ctx.parent_elements));

            is_valid = false;
          }
        };
      }
    }

    is_valid
  }
}

impl<K, V> From<MapValidator<K, V>> for ProtoOption
where
  K: ProtoValidator,
  V: ProtoValidator,
{
  fn from(validator: MapValidator<K, V>) -> Self {
    let mut rules = OptionMessageBuilder::new();

    rules
      .maybe_set("min_pairs", validator.min_pairs)
      .maybe_set("max_pairs", validator.max_pairs);

    if let Some(keys_option) = validator.keys {
      let keys_schema: Self = keys_option.into();

      rules.set("keys", keys_schema.value);
    }

    if let Some(values_option) = validator.values {
      let values_schema: Self = values_option.into();

      rules.set("values", values_schema.value);
    }

    let mut outer_rules = OptionMessageBuilder::new();

    outer_rules.set("map", OptionValue::Message(rules.into()));

    outer_rules
      .add_cel_options(validator.cel)
      .set_ignore(validator.ignore);

    Self {
      name: "(buf.validate.field)".into(),
      value: OptionValue::Message(outer_rules.into()),
    }
  }
}
