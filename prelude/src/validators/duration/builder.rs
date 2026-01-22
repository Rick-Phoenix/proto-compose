#[doc(hidden)]
pub mod state;
use crate::validators::*;
pub(crate) use state::*;

use proto_types::Duration;

impl ProtoValidator for Duration {
  type Target = Self;
  type Stored = Self;
  type Validator = DurationValidator;
  type Builder = DurationValidatorBuilder;

  type UniqueStore<'a>
    = CopyHybridStore<Self>
  where
    Self: 'a;

  #[inline]
  fn make_unique_store<'a>(_: &Self::Validator, cap: usize) -> Self::UniqueStore<'a> {
    CopyHybridStore::default_with_capacity(cap)
  }
}
impl<S: State> ValidatorBuilderFor<Duration> for DurationValidatorBuilder<S> {
  type Target = Duration;
  type Validator = DurationValidator;
  #[inline]
  fn build_validator(self) -> DurationValidator {
    self.build()
  }
}

#[derive(Clone, Debug)]
pub struct DurationValidatorBuilder<S: State = Empty> {
  _state: PhantomData<S>,

  data: DurationValidator,
}

impl<S: State> Default for DurationValidatorBuilder<S> {
  #[inline]
  fn default() -> Self {
    Self {
      _state: PhantomData,
      data: DurationValidator::default(),
    }
  }
}

impl DurationValidator {
  #[must_use]
  #[inline]
  pub fn builder() -> DurationValidatorBuilder {
    DurationValidatorBuilder::default()
  }
}

impl<S: State> From<DurationValidatorBuilder<S>> for ProtoOption {
  fn from(value: DurationValidatorBuilder<S>) -> Self {
    value.build().into()
  }
}

#[allow(
  clippy::must_use_candidate,
  clippy::use_self,
  clippy::return_self_not_must_use
)]
impl<S: State> DurationValidatorBuilder<S> {
  custom_error_messages_method!(Duration);

  #[inline]
  pub fn cel(mut self, program: CelProgram) -> DurationValidatorBuilder<S> {
    self.data.cel.push(program);

    DurationValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn ignore_always(mut self) -> DurationValidatorBuilder<SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    self.data.ignore = Ignore::Always;

    DurationValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn required(mut self) -> DurationValidatorBuilder<SetRequired<S>>
  where
    S::Required: IsUnset,
  {
    self.data.required = true;

    DurationValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn in_(mut self, val: impl IntoSortedList<Duration>) -> DurationValidatorBuilder<SetIn<S>>
  where
    S::In: IsUnset,
  {
    self.data.in_ = Some(val.into_sorted_list());

    DurationValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn not_in(
    mut self,
    val: impl IntoSortedList<Duration>,
  ) -> DurationValidatorBuilder<SetNotIn<S>>
  where
    S::NotIn: IsUnset,
  {
    self.data.not_in = Some(val.into_sorted_list());

    DurationValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn const_(mut self, val: Duration) -> DurationValidatorBuilder<SetConst<S>>
  where
    S::Const: IsUnset,
  {
    self.data.const_ = Some(val);

    DurationValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn lt(mut self, val: Duration) -> DurationValidatorBuilder<SetLt<S>>
  where
    S::Lt: IsUnset,
  {
    self.data.lt = Some(val);

    DurationValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn lte(mut self, val: Duration) -> DurationValidatorBuilder<SetLte<S>>
  where
    S::Lte: IsUnset,
  {
    self.data.lte = Some(val);

    DurationValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn gt(mut self, val: Duration) -> DurationValidatorBuilder<SetGt<S>>
  where
    S::Gt: IsUnset,
  {
    self.data.gt = Some(val);

    DurationValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn gte(mut self, val: Duration) -> DurationValidatorBuilder<SetGte<S>>
  where
    S::Gte: IsUnset,
  {
    self.data.gte = Some(val);

    DurationValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn build(self) -> DurationValidator {
    self.data
  }
}
