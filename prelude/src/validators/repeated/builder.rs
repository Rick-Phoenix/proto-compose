#[doc(hidden)]
pub mod state;
use crate::validators::*;
pub(crate) use state::*;

#[derive(Clone, Debug)]
pub struct RepeatedValidatorBuilder<T, S: State = Empty>
where
  T: ProtoValidator,
{
  _state: PhantomData<S>,
  _inner_type: PhantomData<T>,

  cel: Vec<CelProgram>,
  /// Specifies the rules that will be applied to the individual items of this repeated field.
  items: Option<T::Validator>,
  /// The minimum amount of items that this field must contain in order to be valid.
  min_items: Option<usize>,
  /// The maximum amount of items that this field must contain in order to be valid.
  max_items: Option<usize>,
  /// Specifies that this field must contain only unique values (only applies to scalar fields).
  unique: bool,
  ignore: Ignore,

  error_messages: Option<ErrorMessages<RepeatedViolation>>,
}

impl<T, S: State> Default for RepeatedValidatorBuilder<T, S>
where
  T: ProtoValidator,
{
  #[inline]
  fn default() -> Self {
    Self {
      _state: PhantomData,
      _inner_type: PhantomData,
      cel: Default::default(),
      items: Default::default(),
      min_items: Default::default(),
      max_items: Default::default(),
      unique: Default::default(),
      ignore: Default::default(),
      error_messages: None,
    }
  }
}

impl<T> RepeatedValidator<T>
where
  T: ProtoValidator,
{
  #[must_use]
  #[inline]
  pub fn builder() -> RepeatedValidatorBuilder<T> {
    RepeatedValidatorBuilder::default()
  }
}

impl<T, S: State> RepeatedValidatorBuilder<T, S>
where
  T: ProtoValidator,
{
  #[inline]
  pub fn build(self) -> RepeatedValidator<T> {
    let Self {
      _inner_type,
      items,
      min_items,
      max_items,
      unique,
      ignore,
      cel,
      error_messages,
      ..
    } = self;

    RepeatedValidator {
      _inner_type,
      cel,
      items: items.or_else(|| T::default_validator()),
      min_items,
      max_items,
      unique,
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

  /// Specifies the rules that will be applied to the individual items of this repeated field.
  #[inline]
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
      cel: self.cel,
      min_items: self.min_items,
      max_items: self.max_items,
      unique: self.unique,
      ignore: self.ignore,
      error_messages: self.error_messages,
    }
  }

  #[inline]
  pub fn ignore_if_zero_value(self) -> RepeatedValidatorBuilder<T, SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    RepeatedValidatorBuilder {
      _state: PhantomData,
      _inner_type: self._inner_type,
      cel: self.cel,
      items: self.items,
      min_items: self.min_items,
      max_items: self.max_items,
      unique: self.unique,
      ignore: Ignore::IfZeroValue,
      error_messages: self.error_messages,
    }
  }

  /// Rules set for this field will always be ignored.
  #[inline]
  pub fn ignore_always(self) -> RepeatedValidatorBuilder<T, SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    RepeatedValidatorBuilder {
      _state: PhantomData,
      _inner_type: self._inner_type,
      cel: self.cel,
      items: self.items,
      min_items: self.min_items,
      max_items: self.max_items,
      unique: self.unique,
      ignore: Ignore::Always,
      error_messages: self.error_messages,
    }
  }

  #[inline]
  pub fn min_items(self, num: usize) -> RepeatedValidatorBuilder<T, SetMinItems<S>>
  where
    S::MinItems: IsUnset,
  {
    RepeatedValidatorBuilder {
      _state: PhantomData,
      _inner_type: self._inner_type,
      cel: self.cel,
      items: self.items,
      min_items: Some(num),
      max_items: self.max_items,
      unique: self.unique,
      ignore: self.ignore,
      error_messages: self.error_messages,
    }
  }

  #[inline]
  pub fn max_items(self, num: usize) -> RepeatedValidatorBuilder<T, SetMaxItems<S>>
  where
    S::MaxItems: IsUnset,
  {
    RepeatedValidatorBuilder {
      _state: PhantomData,
      _inner_type: self._inner_type,
      cel: self.cel,
      items: self.items,
      min_items: self.min_items,
      max_items: Some(num),
      unique: self.unique,
      ignore: self.ignore,
      error_messages: self.error_messages,
    }
  }

  #[inline]
  pub fn unique(self) -> RepeatedValidatorBuilder<T, SetUnique<S>>
  where
    S::Unique: IsUnset,
  {
    RepeatedValidatorBuilder {
      _state: PhantomData,
      _inner_type: self._inner_type,
      cel: self.cel,
      items: self.items,
      min_items: self.min_items,
      max_items: self.max_items,
      unique: true,
      ignore: self.ignore,
      error_messages: self.error_messages,
    }
  }
}

impl<T, S: State> From<RepeatedValidatorBuilder<T, S>> for ProtoOption
where
  T: ProtoValidator,
{
  fn from(value: RepeatedValidatorBuilder<T, S>) -> Self {
    value.build().into()
  }
}
