use float_eq::float_eq;
use std::marker::PhantomData;

use float_eq::FloatEq;
use proto_types::protovalidate::field_path_element::Subscript;
use protocheck_core::ordered_float::FloatCore;
use repeated_validator_builder::{SetIgnore, SetItems, SetMaxItems, SetMinItems, SetUnique, State};

use super::{builder_internals::*, *};

impl<T: AsProtoField> AsProtoField for Vec<T> {
  fn as_proto_field() -> ProtoFieldInfo {
    let inner_type = T::as_proto_field();

    match inner_type {
      ProtoFieldInfo::Single(typ) => ProtoFieldInfo::Repeated(typ),
      _ => panic!("Repeated fields cannot be optional, maps or other repeated fields",),
    }
  }
}

impl<T> ProtoValidator for Vec<T>
where
  T: AsProtoType + ProtoValidator,
{
  type Target = Vec<T::Target>;
  type Validator = RepeatedValidator<T>;
  type Builder = RepeatedValidatorBuilder<T>;

  fn validator_builder() -> Self::Builder {
    RepeatedValidator::builder()
  }
}

impl<T, S> ValidatorBuilderFor<Vec<T>> for RepeatedValidatorBuilder<T, S>
where
  S: State,
  T: AsProtoType + ProtoValidator,
{
  type Target = Vec<T::Target>;
  type Validator = RepeatedValidator<T>;

  fn build_validator(self) -> Self::Validator {
    self.build()
  }
}

fn clamp_capacity_for_unique_items_collection<T>(requested_cap: usize) -> usize {
  // 128KB Budget
  const MAX_BYTES: usize = 128 * 1024;
  let item_size = std::mem::size_of::<T>();

  // For ZSTs, uniqueness checks would fail after one insertion anyway
  if item_size == 0 {
    return 1;
  }

  let max_items = MAX_BYTES / item_size;

  requested_cap.min(max_items)
}

pub trait UniqueStore<'a> {
  type Item: ?Sized;

  fn default_with_capacity(cap: usize) -> Self;
  fn insert(&mut self, item: &'a Self::Item) -> bool;
}

// Just for checking uniqueness for messages
pub struct LinearRefStore<'a, T>
where
  T: 'a + ?Sized,
{
  seen: Vec<&'a T>,
}

impl<'a, T> UniqueStore<'a> for LinearRefStore<'a, T>
where
  T: 'a + PartialEq + ?Sized,
{
  type Item = T;

  fn default_with_capacity(cap: usize) -> Self {
    let clamped_cap = clamp_capacity_for_unique_items_collection::<&T>(cap);

    Self {
      seen: Vec::with_capacity(clamped_cap),
    }
  }

  fn insert(&mut self, item: &'a T) -> bool {
    if self.seen.contains(&item) {
      false
    } else {
      self.seen.push(item);
      true
    }
  }
}

#[derive(Default)]
pub struct FloatEpsilonStore<T>
where
  T: FloatCore + FloatEq<Tol = T>,
{
  seen: Vec<OrderedFloat<T>>,
  abs_tol: T,
  rel_tol: T,
}

impl<T> FloatEpsilonStore<T>
where
  T: FloatCore + FloatEq<Tol = T>,
{
  pub fn new(cap: usize, abs: T, rel: T) -> Self {
    let clamped_cap = clamp_capacity_for_unique_items_collection::<T>(cap);

    Self {
      seen: Vec::with_capacity(clamped_cap),
      abs_tol: abs,
      rel_tol: rel,
    }
  }

  pub fn check_neighbors(&self, idx: usize, item: T) -> bool {
    // Idx at insertion point
    if let Some(above) = self.seen.get(idx)
      && float_eq!(above.0, item, abs <= self.abs_tol, r2nd <= self.rel_tol)
    {
      return true;
    }

    // Idx before insertion point
    if idx > 0
      && let Some(below) = self.seen.get(idx - 1)
      && float_eq!(below.0, item, abs <= self.abs_tol, r2nd <= self.rel_tol)
    {
      return true;
    }

    false
  }
}

impl<'a, T> UniqueStore<'a> for FloatEpsilonStore<T>
where
  T: FloatCore + FloatEq<Tol = T> + Default + 'a,
{
  type Item = T;

  fn default_with_capacity(cap: usize) -> Self {
    let clamped_cap = clamp_capacity_for_unique_items_collection::<T>(cap);

    Self {
      seen: Vec::with_capacity(clamped_cap),
      abs_tol: Default::default(),
      rel_tol: Default::default(),
    }
  }

  fn insert(&mut self, item: &Self::Item) -> bool {
    let wrapped = OrderedFloat(*item);

    match self.seen.binary_search(&wrapped) {
      // Exact bit-for-bit match found
      Ok(_) => false,

      // No exact match. 'idx' is the insertion point.
      Err(idx) => {
        let is_duplicate = self.check_neighbors(idx, *item);

        if is_duplicate {
          false
        } else {
          self.seen.insert(idx, wrapped);
          true
        }
      }
    }
  }
}

#[derive(Clone, Debug)]
pub struct RepeatedValidator<T>
where
  T: AsProtoType + ProtoValidator,
{
  _inner_type: PhantomData<T>,

  pub items: Option<T::Validator>,
  /// The minimum amount of items that this field must contain in order to be valid.
  pub min_items: Option<usize>,
  /// The maximum amount of items that this field must contain in order to be valid.
  pub max_items: Option<usize>,
  /// Specifies that this field must contain only unique values (only applies to scalar fields).
  pub unique: bool,
  pub ignore: Option<Ignore>,
}

pub struct UnsupportedStore<T> {
  _marker: PhantomData<T>,
}

impl<T> Default for UnsupportedStore<T> {
  fn default() -> Self {
    Self {
      _marker: PhantomData,
    }
  }
}

impl<'a, T> UniqueStore<'a> for UnsupportedStore<T> {
  type Item = T;

  fn default_with_capacity(_size: usize) -> Self {
    Self::default()
  }

  fn insert(&mut self, _item: &'a Self::Item) -> bool {
    true
  }
}

pub enum RefHybridStore<'a, T>
where
  T: 'a + ?Sized,
{
  Small(Vec<&'a T>),
  Large(HashSet<&'a T>),
}

impl<'a, T> UniqueStore<'a> for RefHybridStore<'a, T>
where
  T: 'a + Eq + Hash + Ord + ?Sized,
{
  type Item = T;

  fn default_with_capacity(cap: usize) -> Self {
    let clamped_cap = clamp_capacity_for_unique_items_collection::<&T>(cap);

    if cap <= 32 {
      Self::Small(Vec::with_capacity(clamped_cap))
    } else {
      Self::Large(HashSet::with_capacity(clamped_cap))
    }
  }

  fn insert(&mut self, item: &'a T) -> bool {
    match self {
      Self::Small(vec) => match vec.binary_search(&item) {
        Ok(_) => false,
        Err(idx) => {
          vec.insert(idx, item);
          true
        }
      },
      Self::Large(set) => set.insert(item),
    }
  }
}

pub enum CopyHybridStore<T> {
  Small(Vec<T>),
  Large(HashSet<T>),
}

impl<'a, T> UniqueStore<'a> for CopyHybridStore<T>
where
  T: 'a + Copy + Eq + Hash + Ord,
{
  type Item = T;

  fn default_with_capacity(cap: usize) -> Self {
    let clamped_cap = clamp_capacity_for_unique_items_collection::<T>(cap);

    if cap <= 32 {
      Self::Small(Vec::with_capacity(clamped_cap))
    } else {
      Self::Large(HashSet::with_capacity(clamped_cap))
    }
  }

  fn insert(&mut self, item: &'a T) -> bool {
    match self {
      Self::Small(vec) => match vec.binary_search(item) {
        Ok(_) => false,
        Err(idx) => {
          vec.insert(idx, *item);
          true
        }
      },
      Self::Large(set) => set.insert(*item),
    }
  }
}

impl<T> Validator<Vec<T>> for RepeatedValidator<T>
where
  T: AsProtoType + ProtoValidator,
{
  type Target = Vec<T::Target>;
  type UniqueStore<'a>
    = UnsupportedStore<Self::Target>
  where
    Self: 'a;

  fn make_unique_store<'a>(&self, _size: usize) -> Self::UniqueStore<'a>
  where
    T: 'a,
  {
    UnsupportedStore::default()
  }

  #[cfg(feature = "testing")]
  fn check_consistency(&self) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if let Err(e) = check_length_rules(
      None,
      length_rule_value!("min_items", self.min_items),
      length_rule_value!("max_items", self.max_items),
    ) {
      errors.push(e);
    }

    if let Some(items_validator) = &self.items
      && let Err(e) = items_validator.check_consistency()
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

    programs.extend(self.items.iter().flat_map(|i| i.cel_programs()));

    programs
  }

  #[cfg(feature = "testing")]
  fn check_cel_programs_with(&self, _val: Self::Target) -> Result<(), Vec<CelError>> {
    if let Some(items_validator) = &self.items {
      items_validator.check_cel_programs()
    } else {
      Ok(())
    }
  }

  fn validate(
    &self,
    field_context: &FieldContext,
    parent_elements: &mut Vec<FieldPathElement>,
    val: Option<&Vec<T::Target>>,
  ) -> Result<(), Violations> {
    handle_ignore_always!(&self.ignore);
    handle_ignore_if_zero_value!(&self.ignore, val.is_none_or(|v| v.is_empty()));

    let mut violations_agg = Violations::new();
    let violations = &mut violations_agg;

    if let Some(val) = val {
      if let Some(min) = &self.min_items
        && val.len() < *min
      {
        violations.add(
          field_context,
          parent_elements,
          &REPEATED_MIN_ITEMS_VIOLATION,
          &format!("must contain at least {min} items"),
        );
      }

      if let Some(max) = &self.max_items
        && val.len() > *max
      {
        violations.add(
          field_context,
          parent_elements,
          &REPEATED_MAX_ITEMS_VIOLATION,
          &format!("cannot contain more than {max} items"),
        );
      }

      let mut items_validator = self
        .items
        .as_ref()
        .filter(|_| !val.is_empty())
        .map(|v| {
          let mut ctx = field_context.clone();
          ctx.field_kind = FieldKind::RepeatedItem;

          (v, ctx)
        });

      // We only create this if there is a `unique` restriction
      let mut unique_store = if self.unique {
        let size = val.len();

        let store = match &self.items {
          Some(v) => v.make_unique_store(size),
          None => {
            <<T as ProtoValidator>::Validator as Validator<T>>::UniqueStore::default_with_capacity(
              size,
            )
          }
        };

        Some(store)
      } else {
        None
      };

      let mut has_unique_values_so_far = true;

      if self.unique || items_validator.is_some() {
        for (i, value) in val.iter().enumerate() {
          if let Some(unique_store) = unique_store.as_mut()
            && has_unique_values_so_far
          {
            has_unique_values_so_far = unique_store.insert(value);
          }

          if let Some((validator, ctx)) = &mut items_validator {
            ctx.subscript = Some(Subscript::Index(i as u64));

            validator
              .validate(ctx, parent_elements, Some(value))
              .ok_or_push_violations(violations);
          }
        }
      }

      if !has_unique_values_so_far {
        violations.add(
          field_context,
          parent_elements,
          &REPEATED_UNIQUE_VIOLATION,
          "must contain unique values",
        );
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
  T: AsProtoType + ProtoValidator,
{
  #[must_use]
  pub const fn builder() -> RepeatedValidatorBuilder<T> {
    RepeatedValidatorBuilder {
      _state: PhantomData,
      _inner_type: PhantomData,
      items: None,
      min_items: None,
      max_items: None,
      unique: false,
      ignore: None,
    }
  }
}

#[derive(Clone, Debug)]
pub struct RepeatedValidatorBuilder<T, S: State = Empty>
where
  T: AsProtoType + ProtoValidator,
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
  pub ignore: Option<Ignore>,
}

impl<T, S: State> RepeatedValidatorBuilder<T, S>
where
  T: AsProtoType + ProtoValidator,
{
  pub fn build(self) -> RepeatedValidator<T> {
    let Self {
      _inner_type,
      items,
      min_items,
      max_items,
      unique,
      ignore,
      ..
    } = self;

    RepeatedValidator {
      _inner_type,
      items,
      min_items,
      max_items,
      unique,
      ignore,
    }
  }

  /// Specifies the rules that will be applied to the individual items of this repeated field.
  pub fn items<F, FinalBuilder>(self, config_fn: F) -> RepeatedValidatorBuilder<T, SetItems<S>>
  where
    S::Items: IsUnset,
    T: ProtoValidator,
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
      ignore: self.ignore,
    }
  }
}

impl<T, S: State> From<RepeatedValidatorBuilder<T, S>> for ProtoOption
where
  T: AsProtoType + ProtoValidator,
{
  fn from(value: RepeatedValidatorBuilder<T, S>) -> Self {
    value.build().into()
  }
}

impl<T> From<RepeatedValidator<T>> for ProtoOption
where
  T: AsProtoType + ProtoValidator,
{
  fn from(validator: RepeatedValidator<T>) -> Self {
    let mut rules: OptionValueList = Vec::new();

    insert_boolean_option!(validator, rules, unique);
    insert_option!(validator, rules, min_items);
    insert_option!(validator, rules, max_items);

    if let Some(items_option) = validator.items {
      let items_schema: Self = items_option.into();

      rules.push((ITEMS.clone(), items_schema.value));
    }

    let mut outer_rules: OptionValueList = vec![];

    outer_rules.push((REPEATED.clone(), OptionValue::Message(rules.into())));

    insert_option!(validator, outer_rules, ignore);

    Self {
      name: BUF_VALIDATE_FIELD.clone(),
      value: OptionValue::Message(outer_rules.into()),
    }
  }
}

#[allow(private_interfaces)]
mod repeated_validator_builder {
  use std::marker::PhantomData;

  use crate::validators::builder_internals::*;

  mod sealed {
    pub(super) struct Sealed;
  }

  mod members {
    pub struct Items;
    pub struct MinItems;
    pub struct MaxItems;
    pub struct Unique;
    pub struct Ignore;
  }

  pub trait State<S = Empty> {
    type Items;
    type MinItems;
    type MaxItems;
    type Unique;
    type Ignore;
    const SEALED: sealed::Sealed;
  }

  pub struct SetItems<S: State = Empty>(PhantomData<fn() -> S>);
  pub struct SetMinItems<S: State = Empty>(PhantomData<fn() -> S>);
  pub struct SetMaxItems<S: State = Empty>(PhantomData<fn() -> S>);
  pub struct SetUnique<S: State = Empty>(PhantomData<fn() -> S>);
  pub struct SetIgnore<S: State = Empty>(PhantomData<fn() -> S>);

  #[doc(hidden)]
  impl State for Empty {
    type Items = Unset<members::Items>;
    type MinItems = Unset<members::MinItems>;
    type MaxItems = Unset<members::MaxItems>;
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
    type Ignore = S::Ignore;
    const SEALED: sealed::Sealed = sealed::Sealed;
  }

  #[doc(hidden)]
  impl<S: State> State for SetUnique<S> {
    type Items = S::Items;
    type MinItems = S::MinItems;
    type MaxItems = S::MaxItems;
    type Unique = Set<members::Unique>;
    type Ignore = S::Ignore;
    const SEALED: sealed::Sealed = sealed::Sealed;
  }
  #[doc(hidden)]
  impl<S: State> State for SetMinItems<S> {
    type Items = S::Items;
    type Unique = S::Unique;
    type MinItems = Set<members::MinItems>;
    type MaxItems = S::MaxItems;
    type Ignore = S::Ignore;
    const SEALED: sealed::Sealed = sealed::Sealed;
  }
  #[doc(hidden)]
  impl<S: State> State for SetMaxItems<S> {
    type Items = S::Items;
    type Unique = S::Unique;
    type MinItems = S::MinItems;
    type MaxItems = Set<members::MaxItems>;
    type Ignore = S::Ignore;
    const SEALED: sealed::Sealed = sealed::Sealed;
  }
  #[doc(hidden)]
  impl<S: State> State for SetIgnore<S> {
    type Items = S::Items;
    type Unique = S::Unique;
    type MinItems = S::MinItems;
    type MaxItems = S::MaxItems;
    type Ignore = Set<members::Ignore>;
    const SEALED: sealed::Sealed = sealed::Sealed;
  }
}
