pub mod state;
use crate::validators::*;
pub use state::*;

#[derive(Debug, Clone)]
pub struct MessageValidatorBuilder<T: ValidatedMessage, S: State = Empty> {
  _state: PhantomData<S>,

  /// Adds custom validation using one or more [`CelRule`]s to this field.
  cel: Vec<CelProgram>,

  ignore: Ignore,

  _message: PhantomData<T>,

  /// Specifies that the field must be set in order to be valid.
  required: bool,
}

impl<T: ValidatedMessage, S: State> Default for MessageValidatorBuilder<T, S> {
  #[inline]
  fn default() -> Self {
    Self {
      _state: PhantomData,
      cel: Default::default(),
      ignore: Default::default(),
      _message: PhantomData,
      required: Default::default(),
    }
  }
}

impl<T: ValidatedMessage> MessageValidator<T> {
  #[must_use]
  #[inline]
  pub fn builder() -> MessageValidatorBuilder<T> {
    MessageValidatorBuilder::default()
  }
}

impl<T: ValidatedMessage, S: State> From<MessageValidatorBuilder<T, S>> for ProtoOption {
  fn from(value: MessageValidatorBuilder<T, S>) -> Self {
    value.build().into()
  }
}

#[allow(
  clippy::must_use_candidate,
  clippy::use_self,
  clippy::return_self_not_must_use
)]
impl<T: ValidatedMessage, S: State> MessageValidatorBuilder<T, S> {
  #[inline]
  pub fn cel(mut self, program: CelProgram) -> MessageValidatorBuilder<T, S> {
    self.cel.push(program);

    MessageValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      _message: self._message,
      required: self.required,
    }
  }

  #[inline]
  pub fn ignore_always(self) -> MessageValidatorBuilder<T, SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    MessageValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: Ignore::Always,
      _message: self._message,
      required: self.required,
    }
  }

  #[inline]
  pub fn required(self) -> MessageValidatorBuilder<T, SetRequired<S>>
  where
    S::Required: IsUnset,
  {
    MessageValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      _message: self._message,
      required: true,
    }
  }

  #[inline]
  pub fn build(self) -> MessageValidator<T> {
    MessageValidator {
      cel: self.cel,
      ignore: self.ignore,
      _message: self._message,
      required: self.required,
    }
  }
}
