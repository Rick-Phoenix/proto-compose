use std::marker::PhantomData;

use proto_types::protovalidate::field_path_element::Subscript;
use repeated_validator_builder::{
  SetCel, SetIgnore, SetItems, SetMaxItems, SetMinItems, SetUnique, State,
};

use super::{builder_internals::*, *};

impl<T: AsProtoField> AsProtoField for Vec<T> {
  fn as_proto_field() -> ProtoFieldInfo {
    let inner_type = T::as_proto_field();

    match inner_type {
      ProtoFieldInfo::Single(typ) => ProtoFieldInfo::Repeated(typ),
      _ => ProtoFieldInfo::Repeated(invalid_type_output(
        "Repeated fields cannot be optional, maps or other repeated fields",
      )),
    }
  }
}

impl<T> ProtoValidator<Vec<T>> for Vec<T>
where
  T: AsProtoType + ProtoValidator<T>,
{
  type Target = Vec<T::Target>;
  type Validator = RepeatedValidator<T>;
  type Builder = RepeatedValidatorBuilder<T>;

  fn builder() -> Self::Builder {
    RepeatedValidator::builder()
  }
}

impl<T, S> ValidatorBuilderFor<Vec<T>> for RepeatedValidatorBuilder<T, S>
where
  S: State,
  T: AsProtoType + ProtoValidator<T>,
{
  type Target = Vec<T::Target>;
  type Validator = RepeatedValidator<T>;

  fn build_validator(self) -> Self::Validator {
    self.build()
  }
}

#[derive(Clone, Debug)]
pub struct RepeatedValidator<T>
where
  T: AsProtoType + ProtoValidator<T>,
{
  _inner_type: PhantomData<T>,

  pub items: Option<T::Validator>,
  /// The minimum amount of items that this field must contain in order to be valid.
  pub min_items: Option<usize>,
  /// The maximum amount of items that this field must contain in order to be valid.
  pub max_items: Option<usize>,
  /// Specifies that this field must contain only unique values (only applies to scalar fields).
  pub unique: bool,
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  /// These will apply to the list as a whole. To apply rules to the individual items, use the items validator instead.
  pub cel: Option<Arc<[CelRule]>>,
  pub ignore: Option<Ignore>,
}

impl<T> Validator<Vec<T>> for RepeatedValidator<T>
where
  T: AsProtoType + ProtoValidator<T>,
{
  type Target = Vec<T::Target>;

  fn cel_rules(&self) -> Option<Arc<[CelRule]>> {
    let items_rules = self.items.as_ref().and_then(|i| i.cel_rules());
    let vec_rules = self.cel.clone();

    if items_rules.is_none() {
      vec_rules
    } else {
      let all_rules: Vec<CelRule> = vec_rules
        .iter()
        .flat_map(|r| r.iter())
        .chain(items_rules.iter().flat_map(|r| r.iter()))
        .cloned()
        .collect();

      Some(all_rules.into())
    }
  }

  fn validate(
    &self,
    field_context: &FieldContext,
    parent_elements: &mut Vec<FieldPathElement>,
    val: Option<&Vec<T::Target>>,
  ) -> Result<(), Vec<Violation>> {
    let mut violations_agg: Vec<Violation> = Vec::new();
    let violations = &mut violations_agg;

    if let Some(val) = val {
      let items_validator = self.items.as_ref().filter(|_| !val.is_empty());

      for (i, value) in val.iter().enumerate() {
        if let Some(validator) = &items_validator {
          let mut ctx = field_context.clone();
          ctx.kind = FieldKind::RepeatedItem;
          ctx.subscript = Some(Subscript::Index(i as u64));

          validator
            .validate(&ctx, parent_elements, Some(value))
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

impl<T> RepeatedValidator<T>
where
  T: AsProtoType + ProtoValidator<T>,
{
  pub fn builder() -> RepeatedValidatorBuilder<T> {
    RepeatedValidatorBuilder {
      _state: PhantomData,
      _inner_type: PhantomData,
      items: None,
      min_items: None,
      max_items: None,
      unique: false,
      cel: None,
      ignore: None,
    }
  }
}

#[derive(Clone, Debug)]
pub struct RepeatedValidatorBuilder<T, S: State = Empty>
where
  T: AsProtoType + ProtoValidator<T>,
{
  _state: PhantomData<S>,
  _inner_type: PhantomData<T>,

  /// Specifies the rules that will be applied to the individual items of this repeated field.
  pub items: Option<T::Validator>,
  /// The minimum amount of items that this field must contain in order to be valid.
  pub min_items: Option<usize>,
  /// The maximum amount of items that this field must contain in order to be valid.
  pub max_items: Option<usize>,
  /// Specifies that this field must contain only unique values (only applies to scalar fields).
  pub unique: bool,
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  /// These will apply to the list as a whole. To apply rules to the individual items, use the items validator instead.
  pub cel: Option<Arc<[CelRule]>>,
  pub ignore: Option<Ignore>,
}

impl<T, S: State> RepeatedValidatorBuilder<T, S>
where
  T: AsProtoType + ProtoValidator<T>,
{
  pub fn build(self) -> RepeatedValidator<T> {
    let Self {
      _inner_type,
      items,
      min_items,
      max_items,
      unique,
      cel,
      ignore,
      ..
    } = self;

    RepeatedValidator {
      _inner_type,
      items,
      min_items,
      max_items,
      unique,
      cel,
      ignore,
    }
  }

  /// Specifies the rules that will be applied to the individual items of this repeated field.
  pub fn items<F, FinalBuilder>(self, config_fn: F) -> RepeatedValidatorBuilder<T, SetItems<S>>
  where
    S::Items: IsUnset,
    T: ProtoValidator<T>,
    FinalBuilder: ValidatorBuilderFor<T, Validator = T::Validator>,
    F: FnOnce(T::Builder) -> FinalBuilder,
  {
    let items_builder = T::validator_from_closure(config_fn);

    RepeatedValidatorBuilder {
      _state: PhantomData,
      _inner_type: self._inner_type,
      items: Some(items_builder),
      min_items: self.min_items,
      max_items: self.max_items,
      unique: self.unique,
      cel: self.cel,
      ignore: self.ignore,
    }
  }

  /// Rules set for this field will always be ignored.
  pub fn ignore_always(self) -> RepeatedValidatorBuilder<T, SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    RepeatedValidatorBuilder {
      _state: PhantomData,
      _inner_type: self._inner_type,
      items: self.items,
      min_items: self.min_items,
      max_items: self.max_items,
      unique: self.unique,
      cel: self.cel,
      ignore: Some(Ignore::Always),
    }
  }

  pub fn min_items(self, num: usize) -> RepeatedValidatorBuilder<T, SetMinItems<S>>
  where
    S::MinItems: IsUnset,
  {
    RepeatedValidatorBuilder {
      _state: PhantomData,
      _inner_type: self._inner_type,
      items: self.items,
      min_items: Some(num),
      max_items: self.max_items,
      unique: self.unique,
      cel: self.cel,
      ignore: self.ignore,
    }
  }

  pub fn max_items(self, num: usize) -> RepeatedValidatorBuilder<T, SetMaxItems<S>>
  where
    S::MaxItems: IsUnset,
  {
    RepeatedValidatorBuilder {
      _state: PhantomData,
      _inner_type: self._inner_type,
      items: self.items,
      min_items: self.min_items,
      max_items: Some(num),
      unique: self.unique,
      cel: self.cel,
      ignore: self.ignore,
    }
  }

  pub fn unique(self) -> RepeatedValidatorBuilder<T, SetUnique<S>>
  where
    S::Unique: IsUnset,
  {
    RepeatedValidatorBuilder {
      _state: PhantomData,
      _inner_type: self._inner_type,
      items: self.items,
      min_items: self.min_items,
      max_items: self.max_items,
      unique: true,
      cel: self.cel,
      ignore: self.ignore,
    }
  }

  pub fn cel(self, rules: impl Into<Arc<[CelRule]>>) -> RepeatedValidatorBuilder<T, SetCel<S>>
  where
    S::Cel: IsUnset,
  {
    RepeatedValidatorBuilder {
      _state: PhantomData,
      _inner_type: self._inner_type,
      items: self.items,
      min_items: self.min_items,
      max_items: self.max_items,
      unique: self.unique,
      cel: Some(rules.into()),
      ignore: self.ignore,
    }
  }
}

impl<T, S: State> From<RepeatedValidatorBuilder<T, S>> for ProtoOption
where
  T: AsProtoType + ProtoValidator<T>,
{
  fn from(value: RepeatedValidatorBuilder<T, S>) -> Self {
    value.build().into()
  }
}

impl<T> From<RepeatedValidator<T>> for ProtoOption
where
  T: AsProtoType + ProtoValidator<T>,
{
  fn from(validator: RepeatedValidator<T>) -> ProtoOption {
    let mut rules: OptionValueList = Vec::new();

    insert_boolean_option!(validator, rules, unique);
    insert_option!(validator, rules, min_items);
    insert_option!(validator, rules, max_items);

    if let Some(items_option) = validator.items {
      let items_schema: ProtoOption = items_option.into();

      rules.push((ITEMS.clone(), items_schema.value));
    }

    let mut outer_rules: OptionValueList = vec![];

    outer_rules.push((REPEATED.clone(), OptionValue::Message(rules.into())));

    insert_cel_rules!(validator, outer_rules);
    insert_option!(validator, outer_rules, ignore);

    ProtoOption {
      name: BUF_VALIDATE_FIELD.clone(),
      value: OptionValue::Message(outer_rules.into()),
    }
  }
}

#[allow(private_interfaces)]
mod repeated_validator_builder {
  use std::marker::PhantomData;

  use crate::validators::builder_internals::*;

  mod members {
    pub struct Items;
    pub struct MinItems;
    pub struct MaxItems;
    pub struct Cel;
    pub struct Unique;
    pub struct Ignore;
  }

  mod sealed {
    pub(super) struct Sealed;
  }

  pub trait State<S = Empty> {
    type Items;
    type MinItems;
    type MaxItems;
    type Unique;
    type Cel;
    type Ignore;
    const SEALED: sealed::Sealed;
  }

  pub struct SetItems<S: State = Empty>(PhantomData<fn() -> S>);
  pub struct SetMinItems<S: State = Empty>(PhantomData<fn() -> S>);
  pub struct SetMaxItems<S: State = Empty>(PhantomData<fn() -> S>);
  pub struct SetUnique<S: State = Empty>(PhantomData<fn() -> S>);
  pub struct SetCel<S: State = Empty>(PhantomData<fn() -> S>);
  pub struct SetIgnore<S: State = Empty>(PhantomData<fn() -> S>);

  #[doc(hidden)]
  impl State for Empty {
    type Items = Unset<members::Items>;
    type MinItems = Unset<members::MinItems>;
    type MaxItems = Unset<members::MaxItems>;
    type Cel = Unset<members::Cel>;
    type Unique = Unset<members::Unique>;
    type Ignore = Unset<members::Ignore>;
    const SEALED: sealed::Sealed = sealed::Sealed;
  }

  #[doc(hidden)]
  impl<S: State> State for SetItems<S> {
    type Items = Set<members::Items>;
    type MinItems = S::MinItems;
    type MaxItems = S::MaxItems;
    type Unique = S::Unique;
    type Cel = S::Cel;
    type Ignore = S::Ignore;
    const SEALED: sealed::Sealed = sealed::Sealed;
  }

  #[doc(hidden)]
  impl<S: State> State for SetUnique<S> {
    type Items = S::Items;
    type MinItems = S::MinItems;
    type MaxItems = S::MaxItems;
    type Unique = Set<members::Unique>;
    type Cel = S::Cel;
    type Ignore = S::Ignore;
    const SEALED: sealed::Sealed = sealed::Sealed;
  }
  #[doc(hidden)]
  impl<S: State> State for SetMinItems<S> {
    type Items = S::Items;
    type Unique = S::Unique;
    type MinItems = Set<members::MinItems>;
    type MaxItems = S::MaxItems;
    type Cel = S::Cel;
    type Ignore = S::Ignore;
    const SEALED: sealed::Sealed = sealed::Sealed;
  }
  #[doc(hidden)]
  impl<S: State> State for SetMaxItems<S> {
    type Items = S::Items;
    type Unique = S::Unique;
    type MinItems = S::MinItems;
    type MaxItems = Set<members::MaxItems>;
    type Cel = S::Cel;
    type Ignore = S::Ignore;
    const SEALED: sealed::Sealed = sealed::Sealed;
  }
  #[doc(hidden)]
  impl<S: State> State for SetCel<S> {
    type Items = S::Items;
    type Unique = S::Unique;
    type MinItems = S::MinItems;
    type MaxItems = S::MaxItems;
    type Cel = Set<members::Cel>;
    type Ignore = S::Ignore;
    const SEALED: sealed::Sealed = sealed::Sealed;
  }
  #[doc(hidden)]
  impl<S: State> State for SetIgnore<S> {
    type Items = S::Items;
    type Unique = S::Unique;
    type MinItems = S::MinItems;
    type MaxItems = S::MaxItems;
    type Cel = S::Cel;
    type Ignore = Set<members::Ignore>;
    const SEALED: sealed::Sealed = sealed::Sealed;
  }
}
