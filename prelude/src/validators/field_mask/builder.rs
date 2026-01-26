#[doc(hidden)]
pub mod state;
use crate::validators::*;
use proto_types::FieldMask;
pub(crate) use state::*;

#[derive(Clone, Debug)]
pub struct FieldMaskValidatorBuilder<S: State = Empty> {
  _state: PhantomData<S>,

  data: FieldMaskValidator,
}

impl<S: State> Default for FieldMaskValidatorBuilder<S> {
  #[inline]
  fn default() -> Self {
    Self {
      _state: PhantomData,
      data: FieldMaskValidator::default(),
    }
  }
}

impl<S: State> ValidatorBuilderFor<FieldMask> for FieldMaskValidatorBuilder<S> {
  type Target = FieldMask;
  type Validator = FieldMaskValidator;
  #[inline]
  fn build_validator(self) -> FieldMaskValidator {
    self.build()
  }
}

impl<S: State> From<FieldMaskValidatorBuilder<S>> for ProtoOption {
  #[inline(never)]
  #[cold]
  fn from(value: FieldMaskValidatorBuilder<S>) -> Self {
    value.build().into()
  }
}

impl FieldMaskValidator {
  #[must_use]
  #[inline]
  pub fn builder() -> FieldMaskValidatorBuilder {
    FieldMaskValidatorBuilder::default()
  }
}

#[allow(
  clippy::must_use_candidate,
  clippy::use_self,
  clippy::return_self_not_must_use
)]
impl<S: State> FieldMaskValidatorBuilder<S> {
  custom_error_messages_method!(FieldMask);

  #[inline]
  pub fn cel(mut self, program: CelProgram) -> FieldMaskValidatorBuilder<S> {
    self.data.cel.push(program);

    FieldMaskValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn ignore_always(mut self) -> FieldMaskValidatorBuilder<SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    self.data.ignore = Ignore::Always;

    FieldMaskValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn required(mut self) -> FieldMaskValidatorBuilder<SetRequired<S>>
  where
    S::Required: IsUnset,
  {
    self.data.required = true;

    FieldMaskValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn in_(mut self, val: impl IntoSortedList<FixedStr>) -> FieldMaskValidatorBuilder<SetIn<S>>
  where
    S::In: IsUnset,
  {
    self.data.in_ = Some(val.into_sorted_list());

    FieldMaskValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn not_in(
    mut self,
    val: impl IntoSortedList<FixedStr>,
  ) -> FieldMaskValidatorBuilder<SetNotIn<S>>
  where
    S::NotIn: IsUnset,
  {
    self.data.not_in = Some(val.into_sorted_list());

    FieldMaskValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn const_(
    mut self,
    val: impl IntoSortedList<FixedStr>,
  ) -> FieldMaskValidatorBuilder<SetConst<S>>
  where
    S::Const: IsUnset,
  {
    self.data.const_ = Some(val.into_sorted_list());

    FieldMaskValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  #[must_use]
  pub fn build(self) -> FieldMaskValidator {
    self.data
  }
}
