#[doc(hidden)]
pub mod state;
use crate::validators::*;
pub(crate) use state::*;

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
  error_messages: Option<ErrorMessages<MapViolation>>,
}

impl<K, V, S: State> Default for MapValidatorBuilder<K, V, S>
where
  K: ProtoValidator,
  V: ProtoValidator,
{
  #[inline]
  fn default() -> Self {
    Self {
      keys: Default::default(),
      _state: PhantomData,
      _key_type: PhantomData,
      _value_type: PhantomData,
      cel: Default::default(),
      values: Default::default(),
      min_pairs: Default::default(),
      max_pairs: Default::default(),
      ignore: Default::default(),
      error_messages: None,
    }
  }
}

impl<K, V> MapValidator<K, V>
where
  K: ProtoValidator,
  V: ProtoValidator,
{
  #[must_use]
  #[inline]
  pub fn builder() -> MapValidatorBuilder<K, V> {
    MapValidatorBuilder::default()
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
  #[inline]
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
      error_messages,
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
      error_messages,
    }
  }

  #[inline]
  #[must_use]
  pub fn cel(mut self, program: CelProgram) -> Self {
    self.cel.push(program);

    self
  }

  #[inline]
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
      error_messages: self.error_messages,
    }
  }

  #[inline]
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
      error_messages: self.error_messages,
    }
  }

  /// Rules set for this field will always be ignored.
  #[inline]
  pub fn ignore_if_zero_value(self) -> MapValidatorBuilder<K, V, SetIgnore<S>>
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
      ignore: Ignore::IfZeroValue,
      error_messages: self.error_messages,
    }
  }

  /// Rules set for this field will always be ignored.
  #[inline]
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
      error_messages: self.error_messages,
    }
  }

  #[inline]
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
      error_messages: self.error_messages,
    }
  }

  #[inline]
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
      error_messages: self.error_messages,
    }
  }
}
