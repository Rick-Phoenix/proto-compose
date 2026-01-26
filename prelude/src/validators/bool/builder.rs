#[doc(hidden)]
pub mod state;
use crate::validators::*;
pub(crate) use state::*;

#[derive(Clone, Debug)]
pub struct BoolValidatorBuilder<S: State = Empty> {
  _state: PhantomData<S>,
  data: BoolValidator,
}

impl ProtoValidation for bool {
  #[doc(hidden)]
  type Target = Self;
  #[doc(hidden)]
  type Stored = Self;
  type Validator = BoolValidator;
  #[doc(hidden)]
  type Builder = BoolValidatorBuilder;

  #[doc(hidden)]
  type UniqueStore<'a>
    = CopyHybridStore<Self>
  where
    Self: 'a;

  #[inline]
  #[doc(hidden)]
  fn make_unique_store<'a>(_: &Self::Validator, _: usize) -> Self::UniqueStore<'a> {
    // This is likely to never be used in the first place, but
    // uniqueness checks would fail after more than 2 elements anyway
    CopyHybridStore::default_with_capacity(2)
  }
}
impl<S: State> ValidatorBuilderFor<bool> for BoolValidatorBuilder<S> {
  type Target = bool;
  type Validator = BoolValidator;
  #[inline]
  fn build_validator(self) -> BoolValidator {
    self.build()
  }
}

impl<S: State> Default for BoolValidatorBuilder<S> {
  #[inline]
  fn default() -> Self {
    Self {
      _state: PhantomData,
      data: BoolValidator::default(),
    }
  }
}

impl BoolValidator {
  #[must_use]
  #[inline]
  pub fn builder() -> BoolValidatorBuilder {
    BoolValidatorBuilder::default()
  }
}

impl<S: State> From<BoolValidatorBuilder<S>> for ProtoOption {
  #[inline(never)]
  #[cold]
  fn from(value: BoolValidatorBuilder<S>) -> Self {
    value.build().into()
  }
}

#[allow(
  clippy::must_use_candidate,
  clippy::use_self,
  clippy::return_self_not_must_use
)]
impl<S: State> BoolValidatorBuilder<S> {
  custom_error_messages_method!(Bool);

  #[inline]
  pub fn ignore_always(mut self) -> BoolValidatorBuilder<SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    self.data.ignore = Ignore::Always;

    BoolValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn ignore_if_zero_value(mut self) -> BoolValidatorBuilder<SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    self.data.ignore = Ignore::IfZeroValue;

    BoolValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn required(mut self) -> BoolValidatorBuilder<SetRequired<S>>
  where
    S::Required: IsUnset,
  {
    self.data.required = true;

    BoolValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn const_(mut self, val: bool) -> BoolValidatorBuilder<SetConst<S>>
  where
    S::Const: IsUnset,
  {
    self.data.const_ = Some(val);

    BoolValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn build(self) -> BoolValidator {
    self.data
  }
}
