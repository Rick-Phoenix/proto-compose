#[doc(hidden)]
pub mod state;
use crate::validators::*;
pub(crate) use state::*;

#[derive(Clone, Debug)]
pub struct EnumValidatorBuilder<T: ProtoEnum, S: State = Empty> {
  _state: PhantomData<S>,
  data: EnumValidator<T>,
}

impl<T: ProtoEnum, S: State> ValidatorBuilderFor<T> for EnumValidatorBuilder<T, S> {
  type Target = i32;
  type Validator = EnumValidator<T>;

  fn build_validator(self) -> Self::Validator {
    self.build()
  }
}

impl<T: ProtoEnum, S: State> Default for EnumValidatorBuilder<T, S> {
  #[inline]
  fn default() -> Self {
    Self {
      _state: PhantomData,
      data: EnumValidator::default(),
    }
  }
}

impl<T: ProtoEnum> EnumValidator<T> {
  #[must_use]
  #[inline]
  pub fn builder() -> EnumValidatorBuilder<T> {
    EnumValidatorBuilder::default()
  }
}

impl<T: ProtoEnum, S: State> From<EnumValidatorBuilder<T, S>> for ProtoOption {
  #[inline(never)]
  #[cold]
  fn from(value: EnumValidatorBuilder<T, S>) -> Self {
    value.build().into()
  }
}

#[allow(
  clippy::must_use_candidate,
  clippy::use_self,
  clippy::return_self_not_must_use
)]
impl<T: ProtoEnum, S: State> EnumValidatorBuilder<T, S> {
  #[inline]
  pub fn with_error_messages(
    mut self,
    error_messages: impl IntoIterator<Item = (EnumViolation, impl Into<FixedStr>)>,
  ) -> EnumValidatorBuilder<T, SetErrorMessages<S>>
  where
    S::ErrorMessages: IsUnset,
  {
    let map: BTreeMap<EnumViolation, FixedStr> = error_messages
      .into_iter()
      .map(|(v, m)| (v, m.into()))
      .collect();
    self.data.error_messages = Some(Box::new(map));

    EnumValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn cel(mut self, program: CelProgram) -> EnumValidatorBuilder<T, S> {
    self.data.cel.push(program);

    EnumValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn ignore_always(mut self) -> EnumValidatorBuilder<T, SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    self.data.ignore = Ignore::Always;

    EnumValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn ignore_if_zero_value(mut self) -> EnumValidatorBuilder<T, SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    self.data.ignore = Ignore::IfZeroValue;

    EnumValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn defined_only(mut self) -> EnumValidatorBuilder<T, SetDefinedOnly<S>>
  where
    S::DefinedOnly: IsUnset,
  {
    self.data.defined_only = true;

    EnumValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn required(mut self) -> EnumValidatorBuilder<T, SetRequired<S>>
  where
    S::Required: IsUnset,
  {
    self.data.required = true;

    EnumValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn in_(mut self, val: impl IntoSortedList<i32>) -> EnumValidatorBuilder<T, SetIn<S>>
  where
    S::In: IsUnset,
  {
    self.data.in_ = Some(val.into_sorted_list());

    EnumValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn not_in(mut self, val: impl IntoSortedList<i32>) -> EnumValidatorBuilder<T, SetNotIn<S>>
  where
    S::NotIn: IsUnset,
  {
    self.data.not_in = Some(val.into_sorted_list());

    EnumValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn const_(mut self, val: i32) -> EnumValidatorBuilder<T, SetConst<S>>
  where
    S::Const: IsUnset,
  {
    self.data.const_ = Some(val);

    EnumValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn build(self) -> EnumValidator<T> {
    self.data
  }
}
