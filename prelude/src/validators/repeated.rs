use std::marker::PhantomData;

use repeated_validator_builder::{
  SetCel, SetIgnore, SetItems, SetMaxItems, SetMinItems, SetUnique, State,
};

use super::{builder_internals::*, *};

macro_rules! impl_repeated {
  ($name:ident) => {
    impl_repeated_validator!($name);
  };
}

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
  type Validator = RepeatedValidator<T, T::Validator>;
  type Builder = RepeatedValidatorBuilder<T, T::Validator>;

  fn builder() -> Self::Builder {
    RepeatedValidator::builder()
  }
}

impl<T, IV, S> ValidatorBuilderFor<Vec<T>> for RepeatedValidatorBuilder<T, IV, S>
where
  S: State,
  T: AsProtoType + ProtoValidator<T, Validator = IV>,
  IV: Validator,
{
  type Validator = RepeatedValidator<T, IV>;

  fn build_validator(self) -> Self::Validator {
    self.build()
  }
}

#[derive(Clone, Debug)]
pub struct RepeatedValidator<T, IV = <T as ProtoValidator<T>>::Validator>
where
  T: AsProtoType + ProtoValidator<T>,
  IV: Validator,
{
  _inner_type: PhantomData<T>,

  pub items: Option<IV>,
  /// The minimum amount of items that this field must contain in order to be valid.
  pub min_items: Option<usize>,
  /// The maximum amount of items that this field must contain in order to be valid.
  pub max_items: Option<usize>,
  /// Specifies that this field must contain only unique values (only applies to scalar fields).
  pub unique: Option<bool>,
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  /// These will apply to the list as a whole. To apply rules to the individual items, use the items validator instead.
  pub cel: Option<Arc<[CelRule]>>,
  pub ignore: Option<Ignore>,
}

impl<T, IV> Validator for RepeatedValidator<T, IV>
where
  T: AsProtoType + ProtoValidator<T>,
  IV: Validator,
{
  type Target = Vec<T>;

  fn validate(&self, val: &Self::Target) -> Result<(), bool> {
    Ok(())
  }
}

impl<T, IV> RepeatedValidator<T, IV>
where
  T: AsProtoType + ProtoValidator<T>,
  IV: Validator,
{
  pub fn builder() -> RepeatedValidatorBuilder<T, IV> {
    RepeatedValidatorBuilder {
      _state: PhantomData,
      _inner_type: PhantomData,
      items: None,
      min_items: None,
      max_items: None,
      unique: None,
      cel: None,
      ignore: None,
    }
  }
}

#[derive(Clone, Debug)]
pub struct RepeatedValidatorBuilder<T, IV = <T as ProtoValidator<T>>::Validator, S: State = Empty>
where
  T: AsProtoType + ProtoValidator<T>,
  IV: Validator,
{
  _state: PhantomData<S>,
  _inner_type: PhantomData<T>,

  /// Specifies the rules that will be applied to the individual items of this repeated field.
  pub items: Option<IV>,
  /// The minimum amount of items that this field must contain in order to be valid.
  pub min_items: Option<usize>,
  /// The maximum amount of items that this field must contain in order to be valid.
  pub max_items: Option<usize>,
  /// Specifies that this field must contain only unique values (only applies to scalar fields).
  pub unique: Option<bool>,
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  /// These will apply to the list as a whole. To apply rules to the individual items, use the items validator instead.
  pub cel: Option<Arc<[CelRule]>>,
  pub ignore: Option<Ignore>,
}

impl<T, IV, S: State> RepeatedValidatorBuilder<T, IV, S>
where
  T: AsProtoType + ProtoValidator<T, Validator = IV>,
  IV: Validator,
{
  pub fn build(self) -> RepeatedValidator<T, IV> {
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
  pub fn items<F, FinalBuilder>(self, config_fn: F) -> RepeatedValidatorBuilder<T, IV, SetItems<S>>
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
  pub fn ignore_always(self) -> RepeatedValidatorBuilder<T, IV, SetIgnore<S>>
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

  pub fn min_items(self, num: usize) -> RepeatedValidatorBuilder<T, IV, SetMinItems<S>>
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

  pub fn max_items(self, num: usize) -> RepeatedValidatorBuilder<T, IV, SetMaxItems<S>>
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

  pub fn unique(self) -> RepeatedValidatorBuilder<T, IV, SetUnique<S>>
  where
    S::Unique: IsUnset,
  {
    RepeatedValidatorBuilder {
      _state: PhantomData,
      _inner_type: self._inner_type,
      items: self.items,
      min_items: self.min_items,
      max_items: self.max_items,
      unique: Some(true),
      cel: self.cel,
      ignore: self.ignore,
    }
  }

  pub fn cel(self, rules: impl Into<Arc<[CelRule]>>) -> RepeatedValidatorBuilder<T, IV, SetCel<S>>
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

impl<T, IV, S: State> From<RepeatedValidatorBuilder<T, IV, S>> for ProtoOption
where
  T: AsProtoType + ProtoValidator<T, Validator = IV>,
  IV: Validator,
{
  fn from(value: RepeatedValidatorBuilder<T, IV, S>) -> Self {
    value.build().into()
  }
}

impl<T, IV> From<RepeatedValidator<T, IV>> for ProtoOption
where
  T: AsProtoType + ProtoValidator<T>,
  IV: Validator,
{
  fn from(validator: RepeatedValidator<T, IV>) -> ProtoOption {
    let mut rules: OptionValueList = Vec::new();

    insert_option!(validator, rules, unique);
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
