mod builder;
use core::hash::BuildHasher;

pub use builder::MapValidatorBuilder;

use proto_types::protovalidate::{
  field_path_element::Subscript, violations_data::map_violations::*,
};

use super::*;

pub trait Map<K, V> {
  fn length(&self) -> usize;

  fn items<'a>(&'a self) -> impl IntoIterator<Item = (&'a K, &'a V)>
  where
    K: 'a,
    V: 'a;

  #[cfg(feature = "cel")]
  fn try_convert_to_cel(self) -> Result<::cel::Value, CelError>
  where
    K: IntoCelKey,
    V: TryIntoCel;
}

impl<K, V> Map<K, V> for BTreeMap<K, V> {
  fn length(&self) -> usize {
    self.len()
  }

  #[cfg(feature = "cel")]
  fn try_convert_to_cel(self) -> Result<::cel::Value, CelError>
  where
    K: IntoCelKey,
    V: TryIntoCel,
  {
    try_convert_to_cel(self)
  }

  fn items<'a>(&'a self) -> impl IntoIterator<Item = (&'a K, &'a V)>
  where
    K: 'a,
    V: 'a,
  {
    self.iter()
  }
}

impl<K, V, S> Map<K, V> for HashMap<K, V, S>
where
  S: BuildHasher,
{
  fn length(&self) -> usize {
    self.len()
  }

  #[cfg(feature = "cel")]
  fn try_convert_to_cel(self) -> Result<::cel::Value, CelError>
  where
    K: IntoCelKey,
    V: TryIntoCel,
  {
    try_convert_to_cel(self)
  }

  fn items<'a>(&'a self) -> impl IntoIterator<Item = (&'a K, &'a V)>
  where
    K: 'a,
    V: 'a,
  {
    self.iter()
  }
}

pub trait ProtoMap<K, V>
where
  K: ProtoValidator,
  V: ProtoValidator,
  K::Stored: Sized,
  V::Stored: Sized,
{
  type Target: Map<K::Stored, V::Stored>;
}

impl<K, V> ProtoMap<K, V> for BTreeMap<K, V>
where
  K: ProtoValidator,
  V: ProtoValidator,
  K::Stored: Sized,
  V::Stored: Sized,
{
  type Target = BTreeMap<K::Stored, V::Stored>;
}

impl<K, V, S> ProtoMap<K, V> for HashMap<K, V, S>
where
  S: BuildHasher,
  K: ProtoValidator,
  V: ProtoValidator,
  K::Stored: Sized,
  V::Stored: Sized,
{
  type Target = HashMap<K::Stored, V::Stored, S>;
}

impl<K, V> ProtoValidator for BTreeMap<K, V>
where
  Self: Clone,
  K: ProtoValidator,
  V: ProtoValidator,
  K::Stored: Sized + Clone + IntoCelKey + Into<Subscript>,
  V::Stored: Sized + Clone + TryIntoCel,
{
  type Target = BTreeMap<K::Stored, V::Stored>;
  type Stored = BTreeMap<K::Stored, V::Stored>;
  type Validator = MapValidator<K, V>;
  type Builder = MapValidatorBuilder<K, V>;

  type UniqueStore<'a>
    = UnsupportedStore<Self::Target>
  where
    Self: 'a;

  const HAS_DEFAULT_VALIDATOR: bool = V::HAS_DEFAULT_VALIDATOR;
}

impl<K, V, S> ProtoValidator for HashMap<K, V, S>
where
  S: BuildHasher + Default + Clone,
  Self: Clone,
  K: ProtoValidator,
  V: ProtoValidator,
  K::Stored: Sized + Clone + IntoCelKey + Into<Subscript>,
  V::Stored: Sized + Clone + TryIntoCel,
{
  type Target = HashMap<K::Stored, V::Stored, S>;
  type Stored = HashMap<K::Stored, V::Stored, S>;
  type Validator = MapValidator<K, V>;
  type Builder = MapValidatorBuilder<K, V>;

  type UniqueStore<'a>
    = UnsupportedStore<Self::Target>
  where
    Self: 'a;

  const HAS_DEFAULT_VALIDATOR: bool = V::HAS_DEFAULT_VALIDATOR;
}

impl<K, V, M, S> ValidatorBuilderFor<M> for MapValidatorBuilder<K, V, S>
where
  S: builder::state::State,
  K: ProtoValidator,
  V: ProtoValidator,
  M: ProtoMap<K, V> + ToOwned,
  M::Target: Clone + Default,
  K::Stored: Sized + Clone + IntoCelKey + Into<Subscript>,
  V::Stored: Sized + Clone + TryIntoCel,
{
  type Target = M::Target;
  type Validator = MapValidator<K, V>;

  fn build_validator(self) -> Self::Validator {
    self.build()
  }
}

impl<K, V, M> Validator<M> for MapValidator<K, V>
where
  K: ProtoValidator,
  V: ProtoValidator,
  M: ProtoMap<K, V> + ToOwned,
  M::Target: Clone + Default,
  K::Stored: Sized + Clone + IntoCelKey + Into<Subscript>,
  V::Stored: Sized + Clone + TryIntoCel,
{
  type Target = M::Target;

  #[cfg(feature = "cel")]
  fn check_cel_programs(&self) -> Result<(), Vec<CelError>> {
    <Self as Validator<M>>::check_cel_programs_with(self, M::Target::default())
  }

  fn check_consistency(&self) -> Result<(), Vec<ConsistencyError>> {
    let mut errors = Vec::new();

    #[cfg(feature = "cel")]
    if let Err(e) = <Self as Validator<M>>::check_cel_programs(self) {
      errors.extend(e.into_iter().map(ConsistencyError::from));
    }

    if let Some(custom_messages) = self.error_messages.as_deref() {
      let mut unused_messages: Vec<String> = Vec::new();

      for key in custom_messages.keys() {
        let is_used = match key {
          MapViolation::MinPairs => self.min_pairs.is_some(),
          MapViolation::MaxPairs => self.max_pairs.is_some(),
          MapViolation::Keys => self.keys.is_some(),
          MapViolation::Values => self.values.is_some(),
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
      match val.try_convert_to_cel() {
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

  fn as_proto_option(&self) -> Option<ProtoOption> {
    Some(self.proto_option())
  }

  fn validate_core<Val>(&self, ctx: &mut ValidationCtx, val: Option<&Val>) -> ValidatorResult
  where
    Val: Borrow<Self::Target> + ?Sized,
  {
    self.validate_map(ctx, val.map(|v| v.borrow()))
  }
}

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

  pub error_messages: Option<ErrorMessages<MapViolation>>,
}

impl<K, V> MapValidator<K, V>
where
  K: ProtoValidator,
  V: ProtoValidator,
{
  pub fn validate_map<M>(&self, ctx: &mut ValidationCtx, val: Option<&M>) -> ValidatorResult
  where
    M: Map<K::Stored, V::Stored> + Clone,
    K::Stored: Sized + Clone + IntoCelKey + Into<Subscript>,
    V::Stored: Sized + Clone + TryIntoCel,
  {
    handle_ignore_always!(&self.ignore);
    handle_ignore_if_zero_value!(&self.ignore, val.is_none_or(|v| v.borrow().length() == 0));

    let mut is_valid = IsValid::Yes;

    if let Some(val) = val {
      let val = val.borrow();

      macro_rules! handle_violation {
        ($id:ident, $default:expr) => {
          is_valid &= ctx.add_map_violation(
            MapViolation::$id,
            self
              .error_messages
              .as_deref()
              .and_then(|map| map.get(&MapViolation::$id))
              .map(|m| Cow::Borrowed(m.as_ref()))
              .unwrap_or_else(|| Cow::Owned($default)),
          )?;
        };
      }

      if let Some(min_pairs) = self.min_pairs
        && val.length() < min_pairs
      {
        handle_violation!(MinPairs, format!("must contain at least {min_pairs} pairs"));
      }

      if let Some(max_pairs) = self.max_pairs
        && val.length() > max_pairs
      {
        handle_violation!(
          MaxPairs,
          format!("cannot contain more than {max_pairs} pairs")
        );
      }

      let keys_validator = self.keys.as_ref();

      let values_validator = self.values.as_ref();

      if keys_validator.is_some() || values_validator.is_some() {
        for (k, v) in val.items() {
          let _ = ctx
            .field_context
            .as_mut()
            .map(|fc| fc.subscript = Some(k.clone().into()));

          if let Some(validator) = keys_validator {
            let _ = ctx
              .field_context
              .as_mut()
              .map(|fc| fc.field_kind = FieldKind::MapKey);

            is_valid &= validator.validate_core(ctx, Some(k))?;
          }

          if let Some(validator) = values_validator {
            let _ = ctx
              .field_context
              .as_mut()
              .map(|fc| fc.field_kind = FieldKind::MapValue);

            is_valid &= validator.validate_core(ctx, Some(v))?;
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
        match val.clone().try_convert_to_cel() {
          Ok(cel_value) => {
            let cel_ctx = ProgramsExecutionCtx {
              programs: &self.cel,
              value: cel_value,
              ctx,
            };

            is_valid &= cel_ctx.execute_programs()?;
          }
          Err(e) => {
            is_valid &= ctx.add_cel_error_violation(e)?;
          }
        };
      }
    }

    Ok(is_valid)
  }

  fn proto_option(&self) -> ProtoOption {
    let mut rules = OptionMessageBuilder::new();

    rules
      .maybe_set("min_pairs", self.min_pairs)
      .maybe_set("max_pairs", self.max_pairs);

    if let Some(keys_option) = self
      .keys
      .as_ref()
      .and_then(|k| k.as_proto_option())
    {
      rules.set("keys", keys_option.value);
    }

    if let Some(values_option) = self
      .values
      .as_ref()
      .and_then(|v| v.as_proto_option())
    {
      rules.set("values", values_option.value);
    }

    let mut outer_rules = OptionMessageBuilder::new();

    if !rules.is_empty() {
      outer_rules.set("map", OptionValue::Message(rules.into()));
    }

    outer_rules
      .add_cel_options(self.cel.clone())
      .set_ignore(self.ignore);

    ProtoOption {
      name: "(buf.validate.field)".into(),
      value: OptionValue::Message(outer_rules.into()),
    }
  }
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

// impl<K, V> ProtoValidator for HashMap<K, V>
// where
//   K: ProtoValidator,
//   V: ProtoValidator,
//   K::Target: Clone + Into<Subscript> + Default + Eq + Hash + IntoCelKey,
//   V::Target: Default + TryIntoCel + Clone,
// {
//   type Target = HashMap<K::Target, V::Target>;
//
//   type Validator = MapValidator<K, V>;
//   type Builder = MapValidatorBuilder<K, V>;
//
//   type UniqueStore<'a>
//     = UnsupportedStore<Self::Target>
//   where
//     Self: 'a;
//
//   const HAS_DEFAULT_VALIDATOR: bool = V::HAS_DEFAULT_VALIDATOR;
// }
//
// impl<K, V> ProtoValidator for BTreeMap<K, V>
// where
//   K: ProtoValidator,
//   V: ProtoValidator,
//   K::Target: Clone + Into<Subscript> + Sized,
//   V::Target: Sized + Clone,
// {
//   type Target = BTreeMap<K::Target, V::Target>;
//
//   type Validator = MapValidator<K, V>;
//   type Builder = MapValidatorBuilder<K, V>;
//
//   type UniqueStore<'a>
//     = UnsupportedStore<Self::Target>
//   where
//     Self: 'a;
//
//   const HAS_DEFAULT_VALIDATOR: bool = V::HAS_DEFAULT_VALIDATOR;
// }

// impl<K, V, S: builder::state::State> ValidatorBuilderFor<BTreeMap<K, V>>
//   for MapValidatorBuilder<K, V, S>
// where
//   K: ProtoValidator,
//   V: ProtoValidator,
//   K::Target: Clone + Into<Subscript> + Sized,
//   V::Target: Sized + Clone,
// {
//   type Target = BTreeMap<K::Target, V::Target>;
//   type Validator = MapValidator<K, V>;
//
//   #[inline]
//   #[doc(hidden)]
//   fn build_validator(self) -> Self::Validator {
//     self.build()
//   }
// }

// impl<K, V, S: builder::state::State> ValidatorBuilderFor<HashMap<K, V>>
//   for MapValidatorBuilder<K, V, S>
// where
//   K: ProtoValidator,
//   V: ProtoValidator,
//   K::Target: Clone + Into<Subscript> + Default + Eq + Hash + IntoCelKey,
//   V::Target: Default + TryIntoCel + Clone,
// {
//   type Target = HashMap<K::Target, V::Target>;
//   type Validator = MapValidator<K, V>;
//
//   #[inline]
//   #[doc(hidden)]
//   fn build_validator(self) -> Self::Validator {
//     self.build()
//   }
// }

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
      values: V::HAS_DEFAULT_VALIDATOR.then(|| V::Validator::default()),
      min_pairs: None,
      max_pairs: None,
      ignore: Ignore::Unspecified,
      error_messages: None,
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
      error_messages: self.error_messages.clone(),
    }
  }
}

#[cfg(feature = "cel")]
fn try_convert_to_cel<K, V>(map: impl IntoIterator<Item = (K, V)>) -> Result<::cel::Value, CelError>
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

// impl<K, V> Validator<BTreeMap<K, V>> for MapValidator<K, V>
// where
//   K: ProtoValidator,
//   V: ProtoValidator,
//   K::Target: Clone + Into<Subscript> + Sized,
//   V::Target: Sized + Clone,
// {
//   type Target = BTreeMap<K::Target, V::Target>;
//
//   #[cfg(feature = "cel")]
//   fn check_cel_programs(&self) -> Result<(), Vec<CelError>> {
//     self.check_cel_programs_with(BTreeMap::default())
//   }
//
//   fn check_consistency(&self) -> Result<(), Vec<ConsistencyError>> {
//     let mut errors = Vec::new();
//
//     if let Err(e) = check_length_rules(
//       None,
//       length_rule_value!("min_pairs", self.min_pairs),
//       length_rule_value!("max_pairs", self.max_pairs),
//     ) {
//       errors.push(e);
//     }
//
//     if let Some(keys_validator) = &self.keys
//       && let Err(e) = keys_validator.check_consistency()
//     {
//       errors.extend(e);
//     }
//
//     if let Some(values_validator) = &self.values
//       && let Err(e) = values_validator.check_consistency()
//     {
//       errors.extend(e);
//     }
//
//     if errors.is_empty() {
//       Ok(())
//     } else {
//       Err(errors)
//     }
//   }
//
//   #[doc(hidden)]
//   fn cel_rules(&self) -> Vec<CelRule> {
//     vec![]
//   }
//
//   #[cfg(feature = "cel")]
//   fn check_cel_programs_with(&self, _val: Self::Target) -> Result<(), Vec<CelError>> {
//     Ok(())
//   }
//
//   fn validate_core<Val>(&self, ctx: &mut ValidationCtx, val: Option<&Val>) -> ValidatorResult
//   where
//     Val: Borrow<Self::Target> + ?Sized,
//   {
//     handle_ignore_always!(&self.ignore);
//     handle_ignore_if_zero_value!(&self.ignore, val.is_none_or(|v| v.borrow().is_empty()));
//
//     let mut is_valid = IsValid::Yes;
//
//     if let Some(val) = val {
//       let val = val.borrow();
//
//       macro_rules! handle_violation {
//         ($id:ident, $default:expr) => {
//           is_valid &= ctx.add_map_violation(
//             MapViolation::$id,
//             self
//               .error_messages
//               .as_deref()
//               .and_then(|map| map.get(&MapViolation::$id))
//               .map(|m| Cow::Borrowed(m.as_ref()))
//               .unwrap_or_else(|| Cow::Owned($default)),
//           )?;
//         };
//       }
//
//       if let Some(min_pairs) = self.min_pairs
//         && val.len() < min_pairs
//       {
//         handle_violation!(
//           MinPairs,
//           format!(
//             "must contain at least {min_pairs} pair{}",
//             pluralize!(min_pairs)
//           )
//         );
//       }
//
//       if let Some(max_pairs) = self.max_pairs
//         && val.len() > max_pairs
//       {
//         handle_violation!(
//           MaxPairs,
//           format!(
//             "cannot contain more than {max_pairs} pair{}",
//             pluralize!(max_pairs)
//           )
//         );
//       }
//
//       let keys_validator = self.keys.as_ref();
//
//       let values_validator = self.values.as_ref();
//
//       if keys_validator.is_some() || values_validator.is_some() {
//         for (k, v) in val {
//           let _ = ctx
//             .field_context
//             .as_mut()
//             .map(|fc| fc.subscript = Some(k.clone().into()));
//
//           if let Some(validator) = keys_validator {
//             let _ = ctx
//               .field_context
//               .as_mut()
//               .map(|fc| fc.field_kind = FieldKind::MapKey);
//
//             is_valid &= validator.validate_core(ctx, Some(k))?;
//           }
//
//           if let Some(validator) = values_validator {
//             let _ = ctx
//               .field_context
//               .as_mut()
//               .map(|fc| fc.field_kind = FieldKind::MapValue);
//
//             is_valid &= validator.validate_core(ctx, Some(v))?;
//           }
//         }
//
//         let _ = ctx
//           .field_context
//           .as_mut()
//           .map(|fc| fc.subscript = None);
//         let _ = ctx
//           .field_context
//           .as_mut()
//           .map(|fc| fc.field_kind = FieldKind::default());
//       }
//     }
//
//     Ok(is_valid)
//   }
//
//   fn as_proto_option(&self) -> Option<ProtoOption> {
//     Some(self.proto_option())
//   }
// }

// impl<K, V> Validator<HashMap<K, V>> for MapValidator<K, V>
// where
//   K: ProtoValidator,
//   V: ProtoValidator,
//   K::Target: Clone + Into<Subscript> + Default + Eq + Hash + IntoCelKey,
//   V::Target: Default + TryIntoCel + Clone,
// {
//   type Target = HashMap<K::Target, V::Target>;
//
//   #[cfg(feature = "cel")]
//   fn check_cel_programs(&self) -> Result<(), Vec<CelError>> {
//     <Self as Validator<HashMap<K, V>>>::check_cel_programs_with(self, HashMap::default())
//   }
//
//   fn check_consistency(&self) -> Result<(), Vec<ConsistencyError>> {
//     let mut errors = Vec::new();
//
//     #[cfg(feature = "cel")]
//     if let Err(e) = <Self as Validator<HashMap<K, V>>>::check_cel_programs(self) {
//       errors.extend(e.into_iter().map(ConsistencyError::from));
//     }
//
//     if let Err(e) = check_length_rules(
//       None,
//       length_rule_value!("min_pairs", self.min_pairs),
//       length_rule_value!("max_pairs", self.max_pairs),
//     ) {
//       errors.push(e);
//     }
//
//     if let Some(keys_validator) = &self.keys
//       && let Err(e) = keys_validator.check_consistency()
//     {
//       errors.extend(e);
//     }
//
//     if let Some(values_validator) = &self.values
//       && let Err(e) = values_validator.check_consistency()
//     {
//       errors.extend(e);
//     }
//
//     if errors.is_empty() {
//       Ok(())
//     } else {
//       Err(errors)
//     }
//   }
//
//   #[doc(hidden)]
//   fn cel_rules(&self) -> Vec<CelRule> {
//     let mut rules: Vec<CelRule> = self.cel.iter().map(|p| p.rule.clone()).collect();
//
//     rules.extend(self.keys.iter().flat_map(|k| k.cel_rules()));
//     rules.extend(self.values.iter().flat_map(|v| v.cel_rules()));
//
//     rules
//   }
//
//   #[cfg(feature = "cel")]
//   fn check_cel_programs_with(&self, val: Self::Target) -> Result<(), Vec<CelError>> {
//     let mut errors: Vec<CelError> = Vec::new();
//
//     if !self.cel.is_empty() {
//       match try_convert_to_cel(val) {
//         Ok(val) => {
//           if let Err(e) = test_programs(&self.cel, val) {
//             errors.extend(e)
//           }
//         }
//         Err(e) => errors.push(e),
//       }
//     }
//
//     if let Some(key_validator) = &self.keys {
//       match key_validator.check_cel_programs() {
//         Ok(()) => {}
//         Err(errs) => errors.extend(errs),
//       };
//     }
//
//     if let Some(values_validator) = &self.values {
//       match values_validator.check_cel_programs() {
//         Ok(()) => {}
//         Err(errs) => errors.extend(errs),
//       };
//     }
//
//     if errors.is_empty() {
//       Ok(())
//     } else {
//       Err(errors)
//     }
//   }
//
//   fn validate_core<Val>(&self, ctx: &mut ValidationCtx, val: Option<&Val>) -> ValidatorResult
//   where
//     Val: Borrow<Self::Target> + ?Sized,
//   {
//     handle_ignore_always!(&self.ignore);
//     handle_ignore_if_zero_value!(&self.ignore, val.is_none_or(|v| v.borrow().is_empty()));
//
//     let mut is_valid = IsValid::Yes;
//
//     if let Some(val) = val {
//       let val = val.borrow();
//
//       macro_rules! handle_violation {
//         ($id:ident, $default:expr) => {
//           is_valid &= ctx.add_map_violation(
//             MapViolation::$id,
//             self
//               .error_messages
//               .as_deref()
//               .and_then(|map| map.get(&MapViolation::$id))
//               .map(|m| Cow::Borrowed(m.as_ref()))
//               .unwrap_or_else(|| Cow::Owned($default)),
//           )?;
//         };
//       }
//
//       if let Some(min_pairs) = self.min_pairs
//         && val.len() < min_pairs
//       {
//         handle_violation!(MinPairs, format!("must contain at least {min_pairs} pairs"));
//       }
//
//       if let Some(max_pairs) = self.max_pairs
//         && val.len() > max_pairs
//       {
//         handle_violation!(
//           MaxPairs,
//           format!("cannot contain more than {max_pairs} pairs")
//         );
//       }
//
//       let keys_validator = self.keys.as_ref();
//
//       let values_validator = self.values.as_ref();
//
//       if keys_validator.is_some() || values_validator.is_some() {
//         for (k, v) in val {
//           let _ = ctx
//             .field_context
//             .as_mut()
//             .map(|fc| fc.subscript = Some(k.clone().into()));
//
//           if let Some(validator) = keys_validator {
//             let _ = ctx
//               .field_context
//               .as_mut()
//               .map(|fc| fc.field_kind = FieldKind::MapKey);
//
//             is_valid &= validator.validate_core(ctx, Some(k))?;
//           }
//
//           if let Some(validator) = values_validator {
//             let _ = ctx
//               .field_context
//               .as_mut()
//               .map(|fc| fc.field_kind = FieldKind::MapValue);
//
//             is_valid &= validator.validate_core(ctx, Some(v))?;
//           }
//         }
//
//         let _ = ctx
//           .field_context
//           .as_mut()
//           .map(|fc| fc.subscript = None);
//         let _ = ctx
//           .field_context
//           .as_mut()
//           .map(|fc| fc.field_kind = FieldKind::default());
//       }
//
//       #[cfg(feature = "cel")]
//       if !self.cel.is_empty() {
//         match try_convert_to_cel(val.clone()) {
//           Ok(cel_value) => {
//             let cel_ctx = ProgramsExecutionCtx {
//               programs: &self.cel,
//               value: cel_value,
//               ctx,
//             };
//
//             is_valid &= cel_ctx.execute_programs()?;
//           }
//           Err(e) => {
//             is_valid &= ctx.add_cel_error_violation(e)?;
//           }
//         };
//       }
//     }
//
//     Ok(is_valid)
//   }
//
//   fn as_proto_option(&self) -> Option<ProtoOption> {
//     Some(self.proto_option())
//   }
// }

impl<K, V> From<MapValidator<K, V>> for ProtoOption
where
  K: ProtoValidator,
  V: ProtoValidator,
{
  fn from(validator: MapValidator<K, V>) -> Self {
    validator.proto_option()
  }
}
