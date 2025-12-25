pub mod state;
use crate::validators::*;
pub use state::*;

#[derive(Clone, Debug, Default)]
pub struct BoolValidatorBuilder<S: State = Empty> {
  _state: PhantomData<S>,
  /// Specifies that only this specific value will be considered valid for this field.
  const_: Option<bool>,
  /// Specifies that the field must be set in order to be valid.
  required: bool,
  ignore: Option<Ignore>,
}

impl BoolValidator {
  #[must_use]
  pub fn builder() -> BoolValidatorBuilder {
    BoolValidatorBuilder::default()
  }
}

impl<S: State> From<BoolValidatorBuilder<S>> for ProtoOption {
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
  pub const fn ignore_always(self) -> BoolValidatorBuilder<SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    BoolValidatorBuilder {
      _state: PhantomData,
      const_: self.const_,
      required: self.required,
      ignore: Some(Ignore::Always),
    }
  }

  pub const fn ignore_if_zero_value(self) -> BoolValidatorBuilder<SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    BoolValidatorBuilder {
      _state: PhantomData,
      const_: self.const_,
      required: self.required,
      ignore: Some(Ignore::IfZeroValue),
    }
  }

  pub const fn required(self) -> BoolValidatorBuilder<SetRequired<S>>
  where
    S::Required: IsUnset,
  {
    BoolValidatorBuilder {
      _state: PhantomData,
      const_: self.const_,
      required: true,
      ignore: self.ignore,
    }
  }

  pub const fn const_(self, val: bool) -> BoolValidatorBuilder<SetConst<S>>
  where
    S::Const: IsUnset,
  {
    BoolValidatorBuilder {
      _state: PhantomData,
      const_: Some(val),
      required: self.required,
      ignore: self.ignore,
    }
  }

  pub const fn build(self) -> BoolValidator {
    BoolValidator {
      const_: self.const_,
      required: self.required,
      ignore: self.ignore,
    }
  }
}
