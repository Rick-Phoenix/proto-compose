pub mod state;
use crate::validators::*;
pub use state::*;

#[derive(Clone, Debug, Default)]
pub struct FieldMaskValidatorBuilder<S: State = Empty> {
  _state: PhantomData<S>,

  /// Adds custom validation using one or more [`CelRule`]s to this field.
  cel: Vec<&'static CelProgram>,

  ignore: Ignore,

  /// Specifies that the field must be set in order to be valid.
  required: bool,

  /// Specifies that only the values in this list will be considered valid for this field.
  in_: Option<&'static StaticLookup<&'static str>>,

  /// Specifies that the values in this list will be considered NOT valid for this field.
  not_in: Option<&'static StaticLookup<&'static str>>,

  /// Specifies that only this specific value will be considered valid for this field.
  const_: Option<&'static StaticLookup<&'static str>>,
}

impl<S: State> From<FieldMaskValidatorBuilder<S>> for ProtoOption {
  fn from(value: FieldMaskValidatorBuilder<S>) -> Self {
    value.build().into()
  }
}

impl FieldMaskValidator {
  #[must_use]
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
  pub fn cel(mut self, program: &'static CelProgram) -> FieldMaskValidatorBuilder<S> {
    self.cel.push(program);

    FieldMaskValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn ignore_always(self) -> FieldMaskValidatorBuilder<SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    FieldMaskValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: Ignore::Always,
      required: self.required,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn required(self) -> FieldMaskValidatorBuilder<SetRequired<S>>
  where
    S::Required: IsUnset,
  {
    FieldMaskValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      required: true,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn in_(self, val: &'static StaticLookup<&'static str>) -> FieldMaskValidatorBuilder<SetIn<S>>
  where
    S::In: IsUnset,
  {
    FieldMaskValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      in_: Some(val),
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn not_in(
    self,
    val: &'static StaticLookup<&'static str>,
  ) -> FieldMaskValidatorBuilder<SetNotIn<S>>
  where
    S::NotIn: IsUnset,
  {
    FieldMaskValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      in_: self.in_,
      not_in: Some(val),
      const_: self.const_,
    }
  }

  pub fn const_(
    self,
    val: &'static StaticLookup<&'static str>,
  ) -> FieldMaskValidatorBuilder<SetConst<S>>
  where
    S::Const: IsUnset,
  {
    FieldMaskValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      in_: self.in_,
      not_in: self.not_in,
      const_: Some(val),
    }
  }

  #[must_use]
  pub fn build(self) -> FieldMaskValidator {
    let Self {
      cel,
      ignore,
      required,
      in_,
      not_in,
      const_,
      ..
    } = self;

    FieldMaskValidator {
      cel,
      ignore,
      required,
      in_,
      not_in,
      const_,
    }
  }
}
