pub mod builder;
pub use builder::RepeatedValidatorBuilder;
use builder::state::State;

use float_eq::float_eq;
use std::marker::PhantomData;

use float_eq::FloatEq;
use proto_types::protovalidate::field_path_element::Subscript;
use protocheck_core::ordered_float::FloatCore;

use super::*;

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
  T::Target: TryIntoCel,
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
  T::Target: TryIntoCel,
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

  pub cel: Vec<&'static CelProgram>,
  pub items: Option<T::Validator>,
  /// The minimum amount of items that this field must contain in order to be valid.
  pub min_items: Option<usize>,
  /// The maximum amount of items that this field must contain in order to be valid.
  pub max_items: Option<usize>,
  /// Specifies that this field must contain only unique values (only applies to scalar fields).
  pub unique: bool,
  pub ignore: Ignore,
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

#[cfg(feature = "cel")]
fn try_convert_to_cel<T: TryIntoCel>(list: Vec<T>) -> Result<::cel::Value, CelError> {
  let values: Vec<::cel::Value> = list
    .into_iter()
    .map(|i| i.try_into_cel())
    .collect::<Result<Vec<::cel::Value>, CelError>>()?;

  Ok(values.into())
}

impl<T> Validator<Vec<T>> for RepeatedValidator<T>
where
  T: AsProtoType + ProtoValidator,
  T::Target: TryIntoCel,
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

    #[cfg(feature = "cel")]
    if let Err(e) = self.check_cel_programs() {
      errors.extend(e.into_iter().map(|e| e.to_string()));
    }

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
    let mut programs = self.cel.clone();

    programs.extend(self.items.iter().flat_map(|i| i.cel_programs()));

    programs
  }

  #[cfg(all(feature = "testing", feature = "cel"))]
  fn check_cel_programs_with(&self, val: Self::Target) -> Result<(), Vec<CelError>> {
    let mut errors = Vec::new();

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

    if let Some(items_validator) = &self.items
      && let Err(e) = items_validator.check_cel_programs()
    {
      errors.extend(e)
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
    val: Option<&Vec<T::Target>>,
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

    if !validator.ignore.is_default() {
      outer_rules.push((IGNORE.clone(), validator.ignore.into()))
    }

    Self {
      name: BUF_VALIDATE_FIELD.clone(),
      value: OptionValue::Message(outer_rules.into()),
    }
  }
}
