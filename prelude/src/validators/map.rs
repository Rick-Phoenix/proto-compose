use std::{collections::HashMap, marker::PhantomData};

use map_validator_builder::{
  SetCel, SetIgnore, SetKeys, SetMaxPairs, SetMinPairs, SetValues, State,
};
use proto_types::protovalidate::{
  field_path_element::Subscript, violations_data::map_violations::*,
};

use super::*;
use crate::field_context::Violations;

pub struct ProtoMap<K, V>(PhantomData<K>, PhantomData<V>);

impl<K: AsProtoField, V: AsProtoField> AsProtoField for ProtoMap<K, V> {
  fn as_proto_field() -> ProtoFieldInfo {
    let keys = match K::as_proto_field() {
      ProtoFieldInfo::Single(data) => data,
      _ => invalid_type_output("Map keys cannot be repeated, optional or nested maps"),
    };

    let values = match V::as_proto_field() {
      ProtoFieldInfo::Single(data) => data,
      _ => invalid_type_output("Map values cannot be repeated, optional or nested maps"),
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
    Subscript::IntKey(self as i64)
  }
}

impl IntoSubscript for u64 {
  fn into_subscript(self) -> Subscript {
    Subscript::UintKey(self)
  }
}

impl<K, V> ProtoValidator<ProtoMap<K, V>> for ProtoMap<K, V>
where
  K: ProtoValidator<K>,
  V: ProtoValidator<V>,
  K::Target: Clone + IntoSubscript + Default + Eq + Hash,
  V::Target: Default,
{
  type Target = HashMap<K::Target, V::Target>;

  type Validator = MapValidator<K, V>;
  type Builder = MapValidatorBuilder<K, V>;

  fn builder() -> Self::Builder {
    MapValidator::builder()
  }
}

impl<K, V, S: State> ValidatorBuilderFor<ProtoMap<K, V>> for MapValidatorBuilder<K, V, S>
where
  K: ProtoValidator<K>,
  V: ProtoValidator<V>,
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
  K: ProtoValidator<K>,
  V: ProtoValidator<V>,
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
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  /// These will apply to the map field as a whole.
  /// To apply cel rules to the individual keys or values, use the validators for those instead.
  pub cel: Option<Arc<[CelRule]>>,
  pub ignore: Option<Ignore>,
}

impl<K, V> Validator<ProtoMap<K, V>> for MapValidator<K, V>
where
  K: ProtoValidator<K>,
  V: ProtoValidator<V>,
  K::Target: Clone + IntoSubscript + Default + Eq + Hash,
  V::Target: Default,
{
  type Target = HashMap<K::Target, V::Target>;

  fn cel_rules(&self) -> Option<Arc<[CelRule]>> {
    let keys_rules = self.keys.as_ref().and_then(|k| k.cel_rules());

    let values_rules = self.values.as_ref().and_then(|v| v.cel_rules());

    let map_rules = self.cel.clone();

    if keys_rules.is_none() && values_rules.is_none() {
      map_rules
    } else {
      let all_rules: Vec<CelRule> = map_rules
        .iter()
        .flat_map(|slice| slice.iter())
        .chain(keys_rules.iter().flat_map(|s| s.iter()))
        .chain(values_rules.iter().flat_map(|s| s.iter()))
        .cloned()
        .collect();

      Some(all_rules.into())
    }
  }

  fn validate_cel(&self) -> Result<(), CelError> {
    if let Some(key_validator) = &self.keys {
      key_validator.validate_cel().unwrap();
    }

    if let Some(values_validator) = &self.values {
      values_validator.validate_cel().unwrap();
    }

    Ok(())
  }

  fn validate(
    &self,
    field_context: &FieldContext,
    parent_elements: &mut Vec<FieldPathElement>,
    val: Option<&HashMap<K::Target, V::Target>>,
  ) -> Result<(), Vec<Violation>> {
    let mut violations_agg: Vec<Violation> = Vec::new();
    let violations = &mut violations_agg;

    if let Some(val) = val {
      if let Some(min_pairs) = self.min_pairs && val.len() < min_pairs {
        violations.add(
          field_context,
          parent_elements,
          &MAP_MIN_PAIRS_VIOLATION,
          &format!("must contain at least {min_pairs} key-value pairs")
        );
      }

      if let Some(max_pairs) = self.max_pairs && val.len() > max_pairs {
        violations.add(
          field_context,
          parent_elements,
          &MAP_MAX_PAIRS_VIOLATION,
          &format!("cannot contain more than {max_pairs} key-value pairs")
        );
      }

      let key_validator = self.keys.as_ref().filter(|_| !val.is_empty());

      let value_validator = self.values.as_ref().filter(|_| !val.is_empty());

      for (k, v) in val {
        if let Some(validator) = &key_validator {
          let mut ctx = field_context.clone();
          ctx.kind = FieldKind::MapKey;
          ctx.subscript = Some(k.clone().into_subscript());

          validator
            .validate(&ctx, parent_elements, Some(k))
            .push_violations(violations);
        }

        if let Some(validator) = &value_validator {
          let mut ctx = field_context.clone();
          ctx.kind = FieldKind::MapValue;
          ctx.subscript = Some(k.clone().into_subscript());

          validator
            .validate(&ctx, parent_elements, Some(v))
            .push_violations(violations);
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
  K: ProtoValidator<K>,
  V: ProtoValidator<V>,
{
  pub fn builder() -> MapValidatorBuilder<K, V> {
    MapValidatorBuilder {
      _state: PhantomData,
      _key_type: PhantomData,
      _value_type: PhantomData,
      values: None,
      keys: None,
      min_pairs: None,
      max_pairs: None,
      cel: None,
      ignore: None,
    }
  }
}

#[derive(Clone, Debug, Default)]
pub struct MapValidatorBuilder<K, V, S: State = Empty>
where
  K: ProtoValidator<K>,
  V: ProtoValidator<V>,
{
  pub keys: Option<K::Validator>,

  _state: PhantomData<S>,
  _key_type: PhantomData<K>,
  _value_type: PhantomData<V>,

  pub values: Option<V::Validator>,
  pub min_pairs: Option<usize>,
  pub max_pairs: Option<usize>,
  pub cel: Option<Arc<[CelRule]>>,
  pub ignore: Option<Ignore>,
}

impl<S: State, K, V> MapValidatorBuilder<K, V, S>
where
  K: ProtoValidator<K>,
  V: ProtoValidator<V>,
{
  pub fn build(self) -> MapValidator<K, V> {
    let Self {
      keys,
      _key_type,
      _value_type,
      values,
      min_pairs,
      max_pairs,
      cel,
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
      cel,
      ignore,
    }
  }

  pub fn cel(self, rules: impl Into<Arc<[CelRule]>>) -> MapValidatorBuilder<K, V, SetCel<S>>
  where
    S::Cel: IsUnset,
  {
    MapValidatorBuilder {
      _state: PhantomData,
      keys: self.keys,
      _key_type: self._key_type,
      _value_type: self._value_type,
      values: self.values,
      min_pairs: self.min_pairs,
      max_pairs: self.max_pairs,
      cel: Some(rules.into()),
      ignore: self.ignore,
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
      cel: self.cel,
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
      cel: self.cel,
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
      cel: self.cel,
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
      cel: self.cel,
      ignore: self.ignore,
    }
  }

  /// Sets the rules for the values of this map field.
  pub fn values<F, FinalBuilder>(self, config_fn: F) -> MapValidatorBuilder<K, V, SetValues<S>>
  where
    V: ProtoValidator<V>,
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
      cel: self.cel,
      ignore: self.ignore,
    }
  }
}

impl<K, V, S: State> From<MapValidatorBuilder<K, V, S>> for ProtoOption
where
  K: ProtoValidator<K>,
  V: ProtoValidator<V>,
{
  fn from(value: MapValidatorBuilder<K, V, S>) -> Self {
    value.build().into()
  }
}

impl<K, V> From<MapValidator<K, V>> for ProtoOption
where
  K: ProtoValidator<K>,
  V: ProtoValidator<V>,
{
  fn from(validator: MapValidator<K, V>) -> Self {
    let mut rules: OptionValueList = Vec::new();

    insert_option!(validator, rules, min_pairs);
    insert_option!(validator, rules, max_pairs);

    if let Some(keys_option) = validator.keys {
      let keys_schema: ProtoOption = keys_option.into();

      rules.push((KEYS.clone(), keys_schema.value));
    }

    if let Some(values_option) = validator.values {
      let values_schema: ProtoOption = values_option.into();

      rules.push((VALUES.clone(), values_schema.value));
    }

    let mut outer_rules: OptionValueList = vec![];

    outer_rules.push((MAP.clone(), OptionValue::Message(rules.into())));

    insert_cel_rules!(validator, outer_rules);
    insert_option!(validator, outer_rules, ignore);

    ProtoOption {
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
    pub struct Cel;
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
    type Cel;
    type Ignore;
    const SEALED: sealed::Sealed;
  }

  pub struct SetKeys<S: State = Empty>(PhantomData<fn() -> S>);
  pub struct SetValues<S: State = Empty>(PhantomData<fn() -> S>);
  pub struct SetMinPairs<S: State = Empty>(PhantomData<fn() -> S>);
  pub struct SetMaxPairs<S: State = Empty>(PhantomData<fn() -> S>);
  pub struct SetCel<S: State = Empty>(PhantomData<fn() -> S>);

  pub struct SetIgnore<S: State = Empty>(PhantomData<fn() -> S>);

  #[doc(hidden)]
  impl State for Empty {
    type Keys = Unset<members::Keys>;
    type Values = Unset<members::Values>;
    type MinPairs = Unset<members::MinPairs>;
    type MaxPairs = Unset<members::MaxPairs>;
    type Cel = Unset<members::Cel>;
    type Ignore = Unset<members::Ignore>;
    const SEALED: sealed::Sealed = sealed::Sealed;
  }

  #[doc(hidden)]
  impl<S: State> State for SetKeys<S> {
    type Keys = Set<members::Keys>;
    type Values = S::Values;
    type MinPairs = S::MinPairs;
    type MaxPairs = S::MaxPairs;
    type Cel = S::Cel;
    type Ignore = S::Ignore;
    const SEALED: sealed::Sealed = sealed::Sealed;
  }

  #[doc(hidden)]
  impl<S: State> State for SetValues<S> {
    type Keys = S::Keys;
    type Values = Set<members::Values>;
    type MinPairs = S::MinPairs;
    type MaxPairs = S::MaxPairs;
    type Cel = S::Cel;
    type Ignore = S::Ignore;
    const SEALED: sealed::Sealed = sealed::Sealed;
  }
  #[doc(hidden)]
  impl<S: State> State for SetMinPairs<S> {
    type Keys = S::Keys;
    type Values = S::Values;
    type MinPairs = Set<members::MinPairs>;
    type MaxPairs = S::MaxPairs;
    type Cel = S::Cel;
    type Ignore = S::Ignore;
    const SEALED: sealed::Sealed = sealed::Sealed;
  }
  #[doc(hidden)]
  impl<S: State> State for SetMaxPairs<S> {
    type Keys = S::Keys;
    type Values = S::Values;
    type MinPairs = S::MinPairs;
    type MaxPairs = Set<members::MaxPairs>;
    type Cel = S::Cel;
    type Ignore = S::Ignore;
    const SEALED: sealed::Sealed = sealed::Sealed;
  }
  #[doc(hidden)]
  impl<S: State> State for SetCel<S> {
    type Keys = S::Keys;
    type Values = S::Values;
    type MinPairs = S::MinPairs;
    type MaxPairs = S::MaxPairs;
    type Cel = Set<members::Cel>;
    type Ignore = S::Ignore;
    const SEALED: sealed::Sealed = sealed::Sealed;
  }
  #[doc(hidden)]
  impl<S: State> State for SetIgnore<S> {
    type Keys = S::Keys;
    type Values = S::Values;
    type MinPairs = S::MinPairs;
    type MaxPairs = S::MaxPairs;
    type Cel = S::Cel;
    type Ignore = Set<members::Ignore>;
    const SEALED: sealed::Sealed = sealed::Sealed;
  }
}
