use std::{collections::HashMap, marker::PhantomData};

use map_validator_builder::{
  SetCel, SetIgnore, SetKeys, SetMaxPairs, SetMinPairs, SetValues, State,
};

use super::*;

pub struct ProtoMap<K, V>(PhantomData<K>, PhantomData<V>);

macro_rules! impl_map {
  ($name:ident) => {
    impl_map_validator!($name);

    impl<K: AsProtoField, V: AsProtoField> AsProtoField for $name<K, V> {
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
  };
}

macro_rules! impl_map_validator {
  ($name:ident) => {
    impl<K, V> ProtoValidator<$name<K, V>> for $name<K, V>
    where
      K: ProtoValidator<K>,
      V: ProtoValidator<V>,
    {
      type Target = HashMap<K::Target, V::Target>;
      type Validator = MapValidator<K, V, K::Validator, V::Validator>;
      type Builder = MapValidatorBuilder<K, V, K::Validator, V::Validator>;

      fn builder() -> Self::Builder {
        MapValidator::builder()
      }
    }

    impl<K, V, KV, VV, S: State> ValidatorBuilderFor<$name<K, V>>
      for MapValidatorBuilder<K, V, KV, VV, S>
    where
      K: ProtoValidator<K, Validator = KV>,
      V: ProtoValidator<V, Validator = VV>,
      KV: Validator<K::Target>,
      VV: Validator<V::Target>,
    {
      type Target = HashMap<K::Target, V::Target>;
      type Validator = MapValidator<K, V, KV, VV>;

      fn build_validator(self) -> Self::Validator {
        self.build()
      }
    }
  };
}

impl_map!(ProtoMap);
impl_map!(HashMap);

#[derive(Clone, Debug)]
pub struct MapValidator<
  K,
  V,
  KV = <K as ProtoValidator<K>>::Validator,
  VV = <V as ProtoValidator<V>>::Builder,
> where
  K: ProtoValidator<K, Validator = KV>,
  V: ProtoValidator<V, Validator = VV>,
  KV: Validator<K::Target>,
  VV: Validator<V::Target>,
{
  /// The validation rules to apply to the keys of this map field.
  pub keys: Option<KV>,

  _key_type: PhantomData<K>,
  _value_type: PhantomData<V>,

  /// The validation rules to apply to the keys of this map field.
  pub values: Option<VV>,
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

impl<K, V, KV, VV> Validator<HashMap<K::Target, V::Target>> for MapValidator<K, V, KV, VV>
where
  K: ProtoValidator<K, Validator = KV>,
  V: ProtoValidator<V, Validator = VV>,
  KV: Validator<K::Target>,
  VV: Validator<V::Target>,
{
  fn validate(&self, val: &HashMap<K::Target, V::Target>) -> Result<(), bool> {
    self.validate(val)
  }
}

impl<K, V, KV, VV> MapValidator<K, V, KV, VV>
where
  K: ProtoValidator<K, Validator = KV>,
  V: ProtoValidator<V, Validator = VV>,
  KV: Validator<K::Target>,
  VV: Validator<V::Target>,
{
  pub fn validate(&self, val: &HashMap<K::Target, V::Target>) -> Result<(), bool> {
    for (k, v) in val {
      if let Some(keys) = &self.keys {
        keys.validate(k);
      }
    }

    Ok(())
  }

  pub fn builder() -> MapValidatorBuilder<K, V, KV, VV> {
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
pub struct MapValidatorBuilder<
  K,
  V,
  KV = <K as ProtoValidator<K>>::Validator,
  VV = <V as ProtoValidator<V>>::Validator,
  S: State = Empty,
> where
  K: ProtoValidator<K, Validator = KV>,
  V: ProtoValidator<V, Validator = VV>,
  KV: Validator<K::Target>,
  VV: Validator<V::Target>,
{
  pub keys: Option<KV>,

  _state: PhantomData<S>,
  _key_type: PhantomData<K>,
  _value_type: PhantomData<V>,

  pub values: Option<VV>,
  pub min_pairs: Option<usize>,
  pub max_pairs: Option<usize>,
  pub cel: Option<Arc<[CelRule]>>,
  pub ignore: Option<Ignore>,
}

impl<S: State, K, V, KV, VV> MapValidatorBuilder<K, V, KV, VV, S>
where
  K: ProtoValidator<K, Validator = KV>,
  V: ProtoValidator<V, Validator = VV>,
  KV: Validator<K::Target>,
  VV: Validator<V::Target>,
{
  pub fn build(self) -> MapValidator<K, V, KV, VV> {
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

  pub fn cel(self, rules: impl Into<Arc<[CelRule]>>) -> MapValidatorBuilder<K, V, KV, VV, SetCel<S>>
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

  pub fn min_pairs(self, num: usize) -> MapValidatorBuilder<K, V, KV, VV, SetMinPairs<S>>
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

  pub fn max_pairs(self, num: usize) -> MapValidatorBuilder<K, V, KV, VV, SetMaxPairs<S>>
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
  pub fn ignore_always(self) -> MapValidatorBuilder<K, V, KV, VV, SetIgnore<S>>
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
  pub fn keys<F, FinalBuilder>(self, config_fn: F) -> MapValidatorBuilder<K, V, KV, VV, SetKeys<S>>
  where
    S::Keys: IsUnset,
    FinalBuilder: ValidatorBuilderFor<K, Validator = KV>,
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
  pub fn values<F, FinalBuilder>(
    self,
    config_fn: F,
  ) -> MapValidatorBuilder<K, V, KV, VV, SetValues<S>>
  where
    V: ProtoValidator<V>,
    FinalBuilder: ValidatorBuilderFor<V, Validator = VV>,
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

impl<K, V, KV, VV, S: State> From<MapValidatorBuilder<K, V, KV, VV, S>> for ProtoOption
where
  K: ProtoValidator<K, Validator = KV>,
  V: ProtoValidator<V, Validator = VV>,
  KV: Validator<K::Target>,
  VV: Validator<V::Target>,
{
  fn from(value: MapValidatorBuilder<K, V, KV, VV, S>) -> Self {
    value.build().into()
  }
}

impl<K, V, KV, VV> From<MapValidator<K, V, KV, VV>> for ProtoOption
where
  K: ProtoValidator<K, Validator = KV>,
  V: ProtoValidator<V, Validator = VV>,
  KV: Validator<K::Target>,
  VV: Validator<V::Target>,
{
  fn from(validator: MapValidator<K, V, KV, VV>) -> Self {
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
