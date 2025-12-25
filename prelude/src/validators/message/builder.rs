pub mod state;
use crate::validators::*;
pub use state::*;

#[derive(Debug, Clone)]
pub struct MessageValidatorBuilder<T: ProtoMessage, S: State = Empty> {
  _state: PhantomData<S>,

  /// Adds custom validation using one or more [`CelRule`]s to this field.
  cel: Vec<&'static CelProgram>,

  ignore: Option<Ignore>,

  _message: PhantomData<T>,

  /// Specifies that the field must be set in order to be valid.
  required: bool,
}

impl<T: ProtoMessage> MessageValidator<T> {
  #[must_use]
  pub const fn builder() -> MessageValidatorBuilder<T> {
    MessageValidatorBuilder {
      _state: PhantomData,
      cel: vec![],
      ignore: None,
      _message: PhantomData,
      required: false,
    }
  }
}

impl<T: ProtoMessage, S: State> From<MessageValidatorBuilder<T, S>> for ProtoOption {
  fn from(value: MessageValidatorBuilder<T, S>) -> Self {
    value.build().into()
  }
}

#[allow(
  clippy::must_use_candidate,
  clippy::use_self,
  clippy::return_self_not_must_use
)]
impl<T: ProtoMessage, S: State> MessageValidatorBuilder<T, S> {
  pub fn cel(mut self, program: &'static CelProgram) -> MessageValidatorBuilder<T, S> {
    self.cel.push(program);

    MessageValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      _message: self._message,
      required: self.required,
    }
  }

  pub fn ignore_always(self) -> MessageValidatorBuilder<T, SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    MessageValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: Some(Ignore::Always),
      _message: self._message,
      required: self.required,
    }
  }

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

  pub fn build(self) -> MessageValidator<T> {
    MessageValidator {
      cel: self.cel,
      ignore: self.ignore,
      _message: self._message,
      required: self.required,
    }
  }
}
