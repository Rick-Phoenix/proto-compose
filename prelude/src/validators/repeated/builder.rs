pub mod state;
use crate::validators::*;
pub use state::*;

#[derive(Clone, Debug)]
pub struct RepeatedValidatorBuilder<T, S: State = Empty>
where
  T: AsProtoType + ProtoValidator,
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
      cel: vec![],
      items: None,
      min_items: None,
      max_items: None,
      unique: false,
      ignore: Ignore::Unspecified,
    }
  }
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
      cel,
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
    }
  }

  pub fn cel(mut self, program: CelProgram) -> RepeatedValidatorBuilder<T, SetCel<S>>
  where
    S::Cel: IsUnset,
  {
    self.cel.push(program);

    RepeatedValidatorBuilder {
      _state: PhantomData,
      _inner_type: self._inner_type,
      items: self.items,
      cel: self.cel,
      min_items: self.min_items,
      max_items: self.max_items,
      unique: self.unique,
      ignore: self.ignore,
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
      cel: self.cel,
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
      cel: self.cel,
      items: self.items,
      min_items: self.min_items,
      max_items: self.max_items,
      unique: self.unique,
      ignore: Ignore::Always,
    }
  }

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
    }
  }

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
    }
  }

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
