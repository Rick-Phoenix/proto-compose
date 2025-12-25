pub mod state;
use crate::validators::*;
pub use state::*;

#[derive(Clone, Debug, Default)]
pub struct AnyValidatorBuilder<S: State = Empty> {
  _state: PhantomData<S>,

  /// Adds custom validation using one or more [`CelRule`]s to this field.
  cel: Vec<&'static CelProgram>,

  ignore: Option<Ignore>,

  /// Specifies that the field must be set in order to be valid.
  required: bool,

  /// Specifies that only the values in this list will be considered valid for this field.
  in_: Option<&'static SortedList<&'static str>>,

  /// Specifies that the values in this list will be considered NOT valid for this field.
  not_in: Option<&'static SortedList<&'static str>>,
}

impl AnyValidator {
  #[must_use]
  pub fn builder() -> AnyValidatorBuilder {
    AnyValidatorBuilder::default()
  }
}

#[allow(
  clippy::must_use_candidate,
  clippy::use_self,
  clippy::return_self_not_must_use
)]
impl<S: State> AnyValidatorBuilder<S> {
  pub fn cel(mut self, program: &'static CelProgram) -> AnyValidatorBuilder<S> {
    self.cel.push(program);

    AnyValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      in_: self.in_,
      not_in: self.not_in,
    }
  }

  pub fn ignore_always(self) -> AnyValidatorBuilder<SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    AnyValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: Some(Ignore::Always),
      required: self.required,
      in_: self.in_,
      not_in: self.not_in,
    }
  }

  pub fn required(self) -> AnyValidatorBuilder<SetRequired<S>>
  where
    S::Required: IsUnset,
  {
    AnyValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      required: true,
      in_: self.in_,
      not_in: self.not_in,
    }
  }

  pub fn in_(self, list: &'static SortedList<&'static str>) -> AnyValidatorBuilder<SetIn<S>>
  where
    S::In: IsUnset,
  {
    AnyValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      in_: Some(list),
      not_in: self.not_in,
    }
  }

  pub fn not_in(self, list: &'static SortedList<&'static str>) -> AnyValidatorBuilder<SetNotIn<S>>
  where
    S::NotIn: IsUnset,
  {
    AnyValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      not_in: Some(list),
      in_: self.in_,
    }
  }

  #[must_use]
  pub fn build(self) -> AnyValidator {
    AnyValidator {
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      in_: self.in_,
      not_in: self.not_in,
    }
  }
}

impl<S: State> From<AnyValidatorBuilder<S>> for ProtoOption {
  fn from(value: AnyValidatorBuilder<S>) -> Self {
    value.build().into()
  }
}
