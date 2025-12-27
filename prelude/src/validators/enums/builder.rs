pub mod state;
use crate::validators::*;
pub use state::*;

#[derive(Clone, Debug, Default)]
pub struct EnumValidatorBuilder<T: ProtoEnum, S: State = Empty> {
  _state: PhantomData<S>,
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  cel: Vec<&'static CelProgram>,

  ignore: Ignore,

  _enum: PhantomData<T>,

  /// Marks that this field will only accept values that are defined in the enum that it's referring to.
  defined_only: bool,

  /// Specifies that the field must be set in order to be valid.
  required: bool,

  /// Specifies that only the values in this list will be considered valid for this field.
  in_: Option<&'static SortedList<i32>>,

  /// Specifies that the values in this list will be considered NOT valid for this field.
  not_in: Option<&'static SortedList<i32>>,

  /// Specifies that only this specific value will be considered valid for this field.
  const_: Option<i32>,
}

impl<T: ProtoEnum> EnumValidator<T> {
  #[must_use]
  pub fn builder() -> EnumValidatorBuilder<T> {
    EnumValidatorBuilder::default()
  }
}

impl<T: ProtoEnum, S: State> From<EnumValidatorBuilder<T, S>> for ProtoOption {
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
  pub fn cel(mut self, program: &'static CelProgram) -> EnumValidatorBuilder<T, S> {
    self.cel.push(program);

    EnumValidatorBuilder {
      _state: PhantomData,
      _enum: self._enum,
      cel: self.cel,
      ignore: self.ignore,
      defined_only: self.defined_only,
      required: self.required,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn ignore_always(self) -> EnumValidatorBuilder<T, SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    EnumValidatorBuilder {
      _state: PhantomData,
      _enum: self._enum,
      cel: self.cel,
      ignore: Ignore::Always,
      defined_only: self.defined_only,
      required: self.required,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn ignore_if_zero_value(self) -> EnumValidatorBuilder<T, SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    EnumValidatorBuilder {
      _state: PhantomData,
      _enum: self._enum,
      cel: self.cel,
      ignore: Ignore::IfZeroValue,
      defined_only: self.defined_only,
      required: self.required,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn defined_only(self) -> EnumValidatorBuilder<T, SetDefinedOnly<S>>
  where
    S::DefinedOnly: IsUnset,
  {
    EnumValidatorBuilder {
      _state: PhantomData,
      _enum: self._enum,
      cel: self.cel,
      ignore: self.ignore,
      defined_only: true,
      required: self.required,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn required(self) -> EnumValidatorBuilder<T, SetRequired<S>>
  where
    S::Required: IsUnset,
  {
    EnumValidatorBuilder {
      _state: PhantomData,
      _enum: self._enum,
      cel: self.cel,
      ignore: self.ignore,
      defined_only: self.defined_only,
      required: true,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn in_(self, val: &'static SortedList<i32>) -> EnumValidatorBuilder<T, SetIn<S>>
  where
    S::In: IsUnset,
  {
    EnumValidatorBuilder {
      _state: PhantomData,
      _enum: self._enum,
      cel: self.cel,
      ignore: self.ignore,
      defined_only: self.defined_only,
      required: self.required,
      in_: Some(val),
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn not_in(self, val: &'static SortedList<i32>) -> EnumValidatorBuilder<T, SetNotIn<S>>
  where
    S::NotIn: IsUnset,
  {
    EnumValidatorBuilder {
      _state: PhantomData,
      _enum: self._enum,
      cel: self.cel,
      ignore: self.ignore,
      defined_only: self.defined_only,
      required: self.required,
      in_: self.in_,
      not_in: Some(val),
      const_: self.const_,
    }
  }

  pub fn const_(self, val: i32) -> EnumValidatorBuilder<T, SetConst<S>>
  where
    S::Const: IsUnset,
  {
    EnumValidatorBuilder {
      _state: PhantomData,
      _enum: self._enum,
      cel: self.cel,
      ignore: self.ignore,
      defined_only: self.defined_only,
      required: self.required,
      in_: self.in_,
      not_in: self.not_in,
      const_: Some(val),
    }
  }

  pub fn build(self) -> EnumValidator<T> {
    EnumValidator {
      cel: self.cel,
      ignore: self.ignore,
      _enum: self._enum,
      defined_only: self.defined_only,
      required: self.required,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }
}
