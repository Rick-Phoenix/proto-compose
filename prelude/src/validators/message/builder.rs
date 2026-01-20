#[doc(hidden)]
pub mod state;
use crate::validators::*;
pub(crate) use state::*;

#[derive(Debug, Clone)]
pub struct MessageValidatorBuilder<S: State = Empty> {
  _state: PhantomData<S>,

  data: MessageValidator,
}

impl<S: State> Default for MessageValidatorBuilder<S> {
  #[inline]
  fn default() -> Self {
    Self {
      _state: PhantomData,
      data: MessageValidator::default(),
    }
  }
}

impl MessageValidator {
  #[must_use]
  #[inline]
  pub fn builder() -> MessageValidatorBuilder {
    MessageValidatorBuilder::default()
  }
}

impl<S: State> From<MessageValidatorBuilder<S>> for ProtoOption {
  fn from(value: MessageValidatorBuilder<S>) -> Self {
    value.build().into()
  }
}

#[allow(
  clippy::must_use_candidate,
  clippy::use_self,
  clippy::return_self_not_must_use
)]
impl<S: State> MessageValidatorBuilder<S> {
  #[inline]
  pub fn cel(mut self, program: CelProgram) -> MessageValidatorBuilder<S> {
    self.data.cel.push(program);

    MessageValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn ignore_always(mut self) -> MessageValidatorBuilder<SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    self.data.ignore = Ignore::Always;

    MessageValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn required(mut self) -> MessageValidatorBuilder<SetRequired<S>>
  where
    S::Required: IsUnset,
  {
    self.data.required = true;

    MessageValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn build(self) -> MessageValidator {
    self.data
  }
}
