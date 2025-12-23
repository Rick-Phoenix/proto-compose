use std::{collections::HashMap, marker::PhantomData};

use map_validator_builder::{SetIgnore, SetKeys, SetMaxPairs, SetMinPairs, SetValues, State};
use proto_types::protovalidate::{
  field_path_element::Subscript, violations_data::map_violations::*,
};

use super::*;
use crate::field_context::ViolationsExt;

pub struct ProtoMap<K, V>(PhantomData<K>, PhantomData<V>);

impl<K: AsProtoField, V: AsProtoField> AsProtoField for ProtoMap<K, V> {
  fn as_proto_field() -> ProtoFieldInfo {
    let keys = match K::as_proto_field() {
      ProtoFieldInfo::Single(data) => data,
      _ => panic!("Map keys cannot be repeated, optional or nested maps"),
    };

    let values = match V::as_proto_field() {
      ProtoFieldInfo::Single(data) => data,
      _ => panic!("Map values cannot be repeated, optional or nested maps"),
    };

    ProtoFieldInfo::Map { keys, values }
  }
}

pub trait IntoSubscript {
  fn into_subscript(self) -> Subscript;
}

impl IntoSubscript for String {
  fn into_subscript(self) -> Subscript {
    Subscript::StringKey(self)
  }
}

impl IntoSubscript for bool {
  fn into_subscript(self) -> Subscript {
    Subscript::BoolKey(self)
  }
}

impl IntoSubscript for i64 {
  fn into_subscript(self) -> Subscript {
    Subscript::IntKey(self)
  }
}

impl IntoSubscript for i32 {
  fn into_subscript(self) -> Subscript {
    Subscript::IntKey(i64::from(self))
  }
}

impl IntoSubscript for u64 {
  fn into_subscript(self) -> Subscript {
    Subscript::UintKey(self)
  }
}

impl<K, V> ProtoValidator for ProtoMap<K, V>
where
  K: ProtoValidator,
  V: ProtoValidator,
  K::Target: Clone + IntoSubscript + Default + Eq + Hash,
  V::Target: Default,
{
  type Target = HashMap<K::Target, V::Target>;

  type Validator = MapValidator<K, V>;
  type Builder = MapValidatorBuilder<K, V>;

  fn validator_builder() -> Self::Builder {
    MapValidator::builder()
  }
}

impl<K, V, S: State> ValidatorBuilderFor<ProtoMap<K, V>> for MapValidatorBuilder<K, V, S>
where
  K: ProtoValidator,
  V: ProtoValidator,
  K::Target: Clone + IntoSubscript + Default + Eq + Hash,
  V::Target: Default,
{
  type Target = HashMap<K::Target, V::Target>;
  type Validator = MapValidator<K, V>;

  fn build_validator(self) -> Self::Validator {
    self.build()
  }
}

#[derive(Clone, Debug)]
pub struct MapValidator<K, V>
where
  K: ProtoValidator,
  V: ProtoValidator,
{
  /// The validation rules to apply to the keys of this map field.
  pub keys: Option<K::Validator>,

  _key_type: PhantomData<K>,
  _value_type: PhantomData<V>,

  /// The validation rules to apply to the keys of this map field.
  pub values: Option<V::Validator>,
  /// The minimum amount of key-value pairs that this field should have in order to be valid.
  pub min_pairs: Option<usize>,
  /// The maximum amount of key-value pairs that this field should have in order to be valid.
  pub max_pairs: Option<usize>,
  pub ignore: Option<Ignore>,
}

impl<K, V> Validator<ProtoMap<K, V>> for MapValidator<K, V>
where
  K: ProtoValidator,
  V: ProtoValidator,
  K::Target: Clone + IntoSubscript + Default + Eq + Hash,
  V::Target: Default,
{
  type Target = HashMap<K::Target, V::Target>;

  #[cfg(feature = "testing")]
  fn check_consistency(&self) -> Result<(), Vec<String>> {
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

  fn cel_rules(&self) -> Vec<&'static CelRule> {
    let mut rules = Vec::new();

    rules.extend(self.keys.iter().flat_map(|k| k.cel_rules()));
    rules.extend(self.values.iter().flat_map(|v| v.cel_rules()));

    rules
  }

  #[cfg(feature = "testing")]
  fn check_cel_programs_with(&self, _val: Self::Target) -> Result<(), Vec<CelError>> {
    let mut errors: Vec<CelError> = Vec::new();

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

  fn validate(
    &self,
    field_context: &FieldContext,
    parent_elements: &mut Vec<FieldPathElement>,
    val: Option<&HashMap<K::Target, V::Target>>,
  ) -> Result<(), Violations> {
    handle_ignore_always!(&self.ignore);
    handle_ignore_if_zero_value!(&self.ignore, val.is_none_or(|v| v.is_empty()));

    let mut violations_agg = Violations::new();
    let violations = &mut violations_agg;

    if let Some(val) = val {
      if let Some(min_pairs) = self.min_pairs
        && val.len() < min_pairs
      {
        violations.add(
          field_context,
          parent_elements,
          &MAP_MIN_PAIRS_VIOLATION,
          &format!("must contain at least {min_pairs} pairs"),
        );
      }

      if let Some(max_pairs) = self.max_pairs
        && val.len() > max_pairs
      {
        violations.add(
          field_context,
          parent_elements,
          &MAP_MAX_PAIRS_VIOLATION,
          &format!("cannot contain more than {max_pairs} pairs"),
        );
      }

      let mut keys_validator = self
        .keys
        .as_ref()
        .filter(|_| !val.is_empty())
        .map(|v| {
          let mut ctx = field_context.clone();
          ctx.field_kind = FieldKind::MapKey;

          (v, ctx)
        });

      let mut values_validator = self
        .values
        .as_ref()
        .filter(|_| !val.is_empty())
        .map(|v| {
          let mut ctx = field_context.clone();
          ctx.field_kind = FieldKind::MapValue;

          (v, ctx)
        });

      if keys_validator.is_some() || values_validator.is_some() {
        for (k, v) in val {
          if let Some((validator, ctx)) = &mut keys_validator {
            ctx.subscript = Some(k.clone().into_subscript());

            validator
              .validate(ctx, parent_elements, Some(k))
              .ok_or_push_violations(violations);
          }

          if let Some((validator, ctx)) = &mut values_validator {
            ctx.subscript = Some(k.clone().into_subscript());

            validator
              .validate(ctx, parent_elements, Some(v))
              .ok_or_push_violations(violations);
          }
        }
      }
    }

    if violations_agg.is_empty() {
      Ok(())
    } else {
      Err(violations_agg)
    }
  }
}

impl<K, V> MapValidator<K, V>
where
  K: ProtoValidator,
  V: ProtoValidator,
{
  #[must_use]
  pub const fn builder() -> MapValidatorBuilder<K, V> {
    MapValidatorBuilder {
      _state: PhantomData,
      _key_type: PhantomData,
      _value_type: PhantomData,
      values: None,
      keys: None,
      min_pairs: None,
      max_pairs: None,
      ignore: None,
    }
  }
}

#[derive(Clone, Debug, Default)]
pub struct MapValidatorBuilder<K, V, S: State = Empty>
where
  K: ProtoValidator,
  V: ProtoValidator,
{
  pub keys: Option<K::Validator>,

  _state: PhantomData<S>,
  _key_type: PhantomData<K>,
  _value_type: PhantomData<V>,

  pub values: Option<V::Validator>,
  pub min_pairs: Option<usize>,
  pub max_pairs: Option<usize>,
  pub ignore: Option<Ignore>,
}

impl<S: State, K, V> MapValidatorBuilder<K, V, S>
where
  K: ProtoValidator,
  V: ProtoValidator,
{
  pub fn build(self) -> MapValidator<K, V> {
    let Self {
      keys,
      _key_type,
      _value_type,
      values,
      min_pairs,
      max_pairs,
      ignore,
      ..
    } = self;

    MapValidator {
      keys,
      _key_type,
      _value_type,
      values,
      min_pairs,
      max_pairs,
      ignore,
    }
  }

  pub fn min_pairs(self, num: usize) -> MapValidatorBuilder<K, V, SetMinPairs<S>>
  where
    S::MinPairs: IsUnset,
  {
    MapValidatorBuilder {
      _state: PhantomData,
      keys: self.keys,
      _key_type: self._key_type,
      _value_type: self._value_type,
      values: self.values,
      min_pairs: Some(num),
      max_pairs: self.max_pairs,
      ignore: self.ignore,
    }
  }

  pub fn max_pairs(self, num: usize) -> MapValidatorBuilder<K, V, SetMaxPairs<S>>
  where
    S::MaxPairs: IsUnset,
  {
    MapValidatorBuilder {
      _state: PhantomData,
      keys: self.keys,
      _key_type: self._key_type,
      _value_type: self._value_type,
      values: self.values,
      min_pairs: self.min_pairs,
      max_pairs: Some(num),
      ignore: self.ignore,
    }
  }

  /// Rules set for this field will always be ignored.
  pub fn ignore_always(self) -> MapValidatorBuilder<K, V, SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    MapValidatorBuilder {
      _state: PhantomData,
      keys: self.keys,
      _key_type: self._key_type,
      _value_type: self._value_type,
      values: self.values,
      min_pairs: self.min_pairs,
      max_pairs: self.max_pairs,
      ignore: Some(Ignore::Always),
    }
  }

  /// Sets the rules for the keys of this map field.
  pub fn keys<F, FinalBuilder>(self, config_fn: F) -> MapValidatorBuilder<K, V, SetKeys<S>>
  where
    S::Keys: IsUnset,
    FinalBuilder: ValidatorBuilderFor<K, Validator = K::Validator>,
    F: FnOnce(K::Builder) -> FinalBuilder,
  {
    let keys_opts = K::validator_from_closure(config_fn);

    MapValidatorBuilder {
      _state: PhantomData,
      keys: Some(keys_opts),
      _key_type: self._key_type,
      _value_type: self._value_type,
      values: self.values,
      min_pairs: self.min_pairs,
      max_pairs: self.max_pairs,
      ignore: self.ignore,
    }
  }

  /// Sets the rules for the values of this map field.
  pub fn values<F, FinalBuilder>(self, config_fn: F) -> MapValidatorBuilder<K, V, SetValues<S>>
  where
    V: ProtoValidator,
    FinalBuilder: ValidatorBuilderFor<V, Validator = V::Validator>,
    F: FnOnce(V::Builder) -> FinalBuilder,
  {
    let values_opts = V::validator_from_closure(config_fn);

    MapValidatorBuilder {
      _state: PhantomData,
      keys: self.keys,
      values: Some(values_opts),
      _key_type: self._key_type,
      _value_type: self._value_type,
      min_pairs: self.min_pairs,
      max_pairs: self.max_pairs,
      ignore: self.ignore,
    }
  }
}

impl<K, V, S: State> From<MapValidatorBuilder<K, V, S>> for ProtoOption
where
  K: ProtoValidator,
  V: ProtoValidator,
{
  fn from(value: MapValidatorBuilder<K, V, S>) -> Self {
    value.build().into()
  }
}

impl<K, V> From<MapValidator<K, V>> for ProtoOption
where
  K: ProtoValidator,
  V: ProtoValidator,
{
  fn from(validator: MapValidator<K, V>) -> Self {
    let mut rules: OptionValueList = Vec::new();

    insert_option!(validator, rules, min_pairs);
    insert_option!(validator, rules, max_pairs);

    if let Some(keys_option) = validator.keys {
      let keys_schema: Self = keys_option.into();

      rules.push((KEYS.clone(), keys_schema.value));
    }

    if let Some(values_option) = validator.values {
      let values_schema: Self = values_option.into();

      rules.push((VALUES.clone(), values_schema.value));
    }

    let mut outer_rules: OptionValueList = vec![];

    outer_rules.push((MAP.clone(), OptionValue::Message(rules.into())));

    insert_option!(validator, outer_rules, ignore);

    Self {
      name: BUF_VALIDATE_FIELD.clone(),
      value: OptionValue::Message(outer_rules.into()),
    }
  }
}

#[allow(private_interfaces)]
mod map_validator_builder {
  use std::marker::PhantomData;

  use crate::validators::builder_internals::*;

  mod members {
    pub struct Keys;
    pub struct Values;
    pub struct MinPairs;
    pub struct MaxPairs;
    pub struct Ignore;
  }

  mod sealed {
    pub(super) struct Sealed;
  }

  pub trait State<S = Empty> {
    type Keys;
    type Values;
    type MinPairs;
    type MaxPairs;
    type Ignore;
    const SEALED: sealed::Sealed;
  }

  pub struct SetKeys<S: State = Empty>(PhantomData<fn() -> S>);
  pub struct SetValues<S: State = Empty>(PhantomData<fn() -> S>);
  pub struct SetMinPairs<S: State = Empty>(PhantomData<fn() -> S>);
  pub struct SetMaxPairs<S: State = Empty>(PhantomData<fn() -> S>);
  pub struct SetIgnore<S: State = Empty>(PhantomData<fn() -> S>);

  #[doc(hidden)]
  impl State for Empty {
    type Keys = Unset<members::Keys>;
    type Values = Unset<members::Values>;
    type MinPairs = Unset<members::MinPairs>;
    type MaxPairs = Unset<members::MaxPairs>;
    type Ignore = Unset<members::Ignore>;
    const SEALED: sealed::Sealed = sealed::Sealed;
  }

  #[doc(hidden)]
  impl<S: State> State for SetKeys<S> {
    type Keys = Set<members::Keys>;
    type Values = S::Values;
    type MinPairs = S::MinPairs;
    type MaxPairs = S::MaxPairs;
    type Ignore = S::Ignore;
    const SEALED: sealed::Sealed = sealed::Sealed;
  }

  #[doc(hidden)]
  impl<S: State> State for SetValues<S> {
    type Keys = S::Keys;
    type Values = Set<members::Values>;
    type MinPairs = S::MinPairs;
    type MaxPairs = S::MaxPairs;
    type Ignore = S::Ignore;
    const SEALED: sealed::Sealed = sealed::Sealed;
  }
  #[doc(hidden)]
  impl<S: State> State for SetMinPairs<S> {
    type Keys = S::Keys;
    type Values = S::Values;
    type MinPairs = Set<members::MinPairs>;
    type MaxPairs = S::MaxPairs;
    type Ignore = S::Ignore;
    const SEALED: sealed::Sealed = sealed::Sealed;
  }
  #[doc(hidden)]
  impl<S: State> State for SetMaxPairs<S> {
    type Keys = S::Keys;
    type Values = S::Values;
    type MinPairs = S::MinPairs;
    type MaxPairs = Set<members::MaxPairs>;
    type Ignore = S::Ignore;
    const SEALED: sealed::Sealed = sealed::Sealed;
  }
  #[doc(hidden)]
  impl<S: State> State for SetIgnore<S> {
    type Keys = S::Keys;
    type Values = S::Values;
    type MinPairs = S::MinPairs;
    type MaxPairs = S::MaxPairs;
    type Ignore = Set<members::Ignore>;
    const SEALED: sealed::Sealed = sealed::Sealed;
  }
}
