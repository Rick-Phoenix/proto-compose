use std::{collections::HashMap, marker::PhantomData};

use map_validator_builder::{IsComplete, IsUnset, SetIgnore, SetKeys, SetValues, State};

use super::*;

pub struct ProtoMap<K, V>(PhantomData<K>, PhantomData<V>);

macro_rules! impl_map {
  ($name:ident) => {
    impl_map_validator!($name);

    impl<K: AsProtoType, V: AsProtoType> AsProtoType for $name<K, V> {
      #[track_caller]
      fn proto_type() -> ProtoType {
        let keys = match K::proto_type() {
          ProtoType::Single(data) => data,
          _ => panic!("Map keys must be scalar types"),
        };

        let values = match V::proto_type() {
          ProtoType::Single(data) => data,
          _ => panic!("Map values cannot be repeated, optional or nested maps"),
        };

        ProtoType::Map { keys, values }
      }
    }
  };
}

macro_rules! impl_map_validator {
  ($name:ident) => {
    impl<K, V> ProtoValidator<$name<K, V>> for ValidatorMap
    where
      ValidatorMap: ProtoValidator<K>,
      ValidatorMap: ProtoValidator<V>,
    {
      type Builder = MapValidatorBuilder<K, V>;

      fn builder() -> Self::Builder {
        MapValidator::builder()
      }
    }

    impl<K, V, S: State> ValidatorBuilderFor<$name<K, V>> for MapValidatorBuilder<K, V, S> {}
  };
}

impl_map!(ProtoMap);
impl_map!(HashMap);

#[derive(Clone, Debug, Builder)]
#[builder(derive(Clone))]
pub struct MapValidator<K = (), V = ()> {
  #[builder(default)]
  _key_type: PhantomData<K>,
  #[builder(default)]
  _value_type: PhantomData<V>,

  /// The validation rules to apply to the keys of this map field.
  #[builder(setters(vis = "", name = keys_internal))]
  pub keys: Option<ProtoOption>,
  /// The validation rules to apply to the keys of this map field.
  #[builder(setters(vis = "", name = values_internal))]
  pub values: Option<ProtoOption>,
  /// The minimum amount of key-value pairs that this field should have in order to be valid.
  pub min_pairs: Option<u64>,
  /// The maximum amount of key-value pairs that this field should have in order to be valid.
  pub max_pairs: Option<u64>,
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  /// These will apply to the map field as a whole.
  /// To apply cel rules to the individual keys or values, use the validators for those instead.
  pub cel: Option<Arc<[CelRule]>>,
  #[builder(with = || true)]
  /// Specifies that the field must be set in order to be valid. This is essentially the same as setting min_pairs to 1.
  pub required: Option<bool>,
  #[builder(setters(vis = "", name = ignore))]
  pub ignore: Option<Ignore>,
}

impl<K, V, S: State> MapValidatorBuilder<K, V, S>
where
  S::Keys: IsUnset,
{
  #[track_caller]
  /// Sets the rules for the keys of this map field.
  pub fn keys<F, FinalBuilder>(self, config_fn: F) -> MapValidatorBuilder<K, V, SetKeys<S>>
  where
    ValidatorMap: ProtoValidator<K>,
    FinalBuilder: ValidatorBuilderFor<K>,
    F: FnOnce(<ValidatorMap as ProtoValidator<K>>::Builder) -> FinalBuilder,
  {
    let keys_opts = ValidatorMap::build_rules(config_fn);
    self.keys_internal(keys_opts)
  }
}

impl<K, V, S: State> MapValidatorBuilder<K, V, S>
where
  S::Values: IsUnset,
{
  #[track_caller]
  /// Sets the rules for the values of this map field.
  pub fn values<F, FinalBuilder>(self, config_fn: F) -> MapValidatorBuilder<K, V, SetValues<S>>
  where
    ValidatorMap: ProtoValidator<V>,
    FinalBuilder: ValidatorBuilderFor<V>,
    F: FnOnce(<ValidatorMap as ProtoValidator<V>>::Builder) -> FinalBuilder,
  {
    let values_opts = ValidatorMap::build_rules(config_fn);
    self.values_internal(values_opts)
  }
}

impl<S: State, K, V> MapValidatorBuilder<K, V, S>
where
  S::Ignore: IsUnset,
{
  /// Rules set for this field will always be ignored.
  pub fn ignore_always(self) -> MapValidatorBuilder<K, V, SetIgnore<S>> {
    self.ignore(Ignore::Always)
  }
}

impl<K, V, S: State> From<MapValidatorBuilder<K, V, S>> for ProtoOption
where
  S: IsComplete,
{
  #[track_caller]
  fn from(value: MapValidatorBuilder<K, V, S>) -> Self {
    value.build().into()
  }
}

impl<KeyItems, ValueItems> From<MapValidator<KeyItems, ValueItems>> for ProtoOption {
  #[track_caller]
  fn from(validator: MapValidator<KeyItems, ValueItems>) -> Self {
    let mut rules: OptionValueList = Vec::new();

    insert_option!(validator, rules, min_pairs);
    insert_option!(validator, rules, max_pairs);

    if let Some(keys_option) = validator.keys {
      rules.push((KEYS.clone(), (keys_option.value).clone()));
    }

    if let Some(values_option) = validator.values {
      rules.push((VALUES.clone(), (values_option.value).clone()));
    }

    let mut outer_rules: OptionValueList = vec![];

    outer_rules.push((MAP.clone(), OptionValue::Message(rules.into())));

    insert_cel_rules!(validator, outer_rules);
    insert_option!(validator, outer_rules, required);
    insert_option!(validator, outer_rules, ignore);

    ProtoOption {
      name: BUF_VALIDATE_FIELD.clone(),
      value: OptionValue::Message(outer_rules.into()),
    }
  }
}
