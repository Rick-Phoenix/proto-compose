pub mod builder;
pub use builder::MapValidatorBuilder;
use builder::state::State;

use std::{collections::HashMap, marker::PhantomData};

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
  K::Target: Clone + IntoSubscript + Default + Eq + Hash + IntoCelKey,
  V::Target: Default + TryIntoCel,
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
  K::Target: Clone + IntoSubscript + Default + Eq + Hash + IntoCelKey,
  V::Target: Default + TryIntoCel,
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

  pub cel: Vec<&'static CelProgram>,

  /// The validation rules to apply to the keys of this map field.
  pub values: Option<V::Validator>,
  /// The minimum amount of key-value pairs that this field should have in order to be valid.
  pub min_pairs: Option<usize>,
  /// The maximum amount of key-value pairs that this field should have in order to be valid.
  pub max_pairs: Option<usize>,
  pub ignore: Ignore,
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

impl<K, V> Validator<ProtoMap<K, V>> for MapValidator<K, V>
where
  K: ProtoValidator,
  V: ProtoValidator,
  K::Target: Clone + IntoSubscript + Default + Eq + Hash + IntoCelKey,
  V::Target: Default + TryIntoCel,
{
  type Target = HashMap<K::Target, V::Target>;
  type UniqueStore<'a>
    = UnsupportedStore<Self::Target>
  where
    Self: 'a;

  fn make_unique_store<'a>(&self, _: usize) -> Self::UniqueStore<'a>
  where
    Self: 'a,
  {
    UnsupportedStore::default()
  }

  #[cfg(feature = "testing")]
  fn check_consistency(&self) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    #[cfg(feature = "cel")]
    if let Err(e) = self.check_cel_programs() {
      errors.extend(e.into_iter().map(|e| e.to_string()));
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

  fn cel_programs(&self) -> Vec<&'static CelProgram> {
    let mut programs = Vec::new();

    programs.extend(self.keys.iter().flat_map(|k| k.cel_programs()));
    programs.extend(self.values.iter().flat_map(|v| v.cel_programs()));

    programs
  }

  #[cfg(all(feature = "testing", feature = "cel"))]
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
      #[cfg(feature = "cel")]
      if !self.cel.is_empty() {
        match try_convert_to_cel(val.clone()) {
          Ok(cel_value) => {
            let ctx = ProgramsExecutionCtx {
              programs: &self.cel,
              value: cel_value,
              violations,
              field_context: Some(field_context),
              parent_elements,
            };

            ctx.execute_programs();
          }
          Err(e) => violations.push(e.into_violation(Some(field_context), parent_elements)),
        };
      }

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

    if !validator.ignore.is_default() {
      outer_rules.push((IGNORE.clone(), validator.ignore.into()))
    }

    Self {
      name: BUF_VALIDATE_FIELD.clone(),
      value: OptionValue::Message(outer_rules.into()),
    }
  }
}
