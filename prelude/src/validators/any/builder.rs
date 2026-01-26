#[doc(hidden)]
pub mod state;
use crate::validators::*;
use proto_types::Any;
pub(crate) use state::*;

#[derive(Clone, Debug)]
pub struct AnyValidatorBuilder<S: State = Empty> {
  _state: PhantomData<S>,

  data: AnyValidator,
}

impl<S: State> Default for AnyValidatorBuilder<S> {
  #[inline]
  fn default() -> Self {
    Self {
      _state: PhantomData,
      data: AnyValidator::default(),
    }
  }
}

impl AnyValidator {
  #[must_use]
  #[inline]
  pub fn builder() -> AnyValidatorBuilder {
    AnyValidatorBuilder::default()
  }
}

impl ProtoValidation for Any {
  #[doc(hidden)]
  type Target = Self;
  #[doc(hidden)]
  type Stored = Self;
  type Validator = AnyValidator;
  #[doc(hidden)]
  type Builder = AnyValidatorBuilder;

  type UniqueStore<'a>
    = LinearRefStore<'a, Self>
  where
    Self: 'a;

  #[inline]
  fn make_unique_store<'a>(_: &Self::Validator, cap: usize) -> Self::UniqueStore<'a> {
    LinearRefStore::default_with_capacity(cap)
  }
}

impl<S: State> ValidatorBuilderFor<Any> for AnyValidatorBuilder<S> {
  type Target = Any;
  type Validator = AnyValidator;
  #[inline]
  fn build_validator(self) -> AnyValidator {
    self.build()
  }
}

#[allow(
  clippy::must_use_candidate,
  clippy::use_self,
  clippy::return_self_not_must_use
)]
impl<S: State> AnyValidatorBuilder<S> {
  custom_error_messages_method!(Any);

  #[inline]
  pub fn cel(mut self, program: CelProgram) -> AnyValidatorBuilder<S> {
    self.data.cel.push(program);

    AnyValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn ignore_always(mut self) -> AnyValidatorBuilder<SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    self.data.ignore = Ignore::Always;

    AnyValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn required(mut self) -> AnyValidatorBuilder<SetRequired<S>>
  where
    S::Required: IsUnset,
  {
    self.data.required = true;

    AnyValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn in_(mut self, list: impl IntoSortedList<FixedStr>) -> AnyValidatorBuilder<SetIn<S>>
  where
    S::In: IsUnset,
  {
    self.data.in_ = Some(list.into_sorted_list());

    AnyValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn not_in(mut self, list: impl IntoSortedList<FixedStr>) -> AnyValidatorBuilder<SetNotIn<S>>
  where
    S::NotIn: IsUnset,
  {
    self.data.not_in = Some(list.into_sorted_list());

    AnyValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[must_use]
  #[inline]
  pub fn build(self) -> AnyValidator {
    self.data
  }
}

impl<S: State> From<AnyValidatorBuilder<S>> for ProtoOption {
  #[inline(never)]
  #[cold]
  fn from(value: AnyValidatorBuilder<S>) -> Self {
    value.build().into()
  }
}
