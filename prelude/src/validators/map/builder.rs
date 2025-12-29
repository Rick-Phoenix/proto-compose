pub mod state;
use crate::validators::*;
pub use state::*;

#[derive(Clone, Debug)]
pub struct MapValidatorBuilder<K, V, S: State = Empty>
where
  K: ProtoValidator,
  V: ProtoValidator,
{
  keys: Option<K::Validator>,

  _state: PhantomData<S>,
  _key_type: PhantomData<K>,
  _value_type: PhantomData<V>,

  cel: Vec<CelProgram>,
  values: Option<V::Validator>,
  min_pairs: Option<usize>,
  max_pairs: Option<usize>,
  ignore: Ignore,
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
      cel: vec![],
      values: None,
      keys: None,
      min_pairs: None,
      max_pairs: None,
      ignore: Ignore::Unspecified,
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
      cel,
      ..
    } = self;

    MapValidator {
      keys,
      _key_type,
      _value_type,
      cel,
      values: values.or_else(|| V::default_validator()),
      min_pairs,
      max_pairs,
      ignore,
    }
  }

  pub fn cel(mut self, program: CelProgram) -> MapValidatorBuilder<K, V, SetCel<S>>
  where
    S::Cel: IsUnset,
  {
    self.cel.push(program);

    MapValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      keys: self.keys,
      _key_type: self._key_type,
      _value_type: self._value_type,
      values: self.values,
      min_pairs: self.min_pairs,
      max_pairs: self.max_pairs,
      ignore: self.ignore,
    }
  }

  pub fn min_pairs(self, num: usize) -> MapValidatorBuilder<K, V, SetMinPairs<S>>
  where
    S::MinPairs: IsUnset,
  {
    MapValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
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
      cel: self.cel,
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
      cel: self.cel,
      keys: self.keys,
      _key_type: self._key_type,
      _value_type: self._value_type,
      values: self.values,
      min_pairs: self.min_pairs,
      max_pairs: self.max_pairs,
      ignore: Ignore::Always,
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
      cel: self.cel,
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
      cel: self.cel,
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
