pub mod state;
use crate::validators::*;
pub use state::*;

use proto_types::Duration;

#[derive(Clone, Debug, Default)]
pub struct DurationValidatorBuilder<S: State = Empty> {
  _state: PhantomData<S>,
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  cel: Vec<&'static CelProgram>,

  ignore: Ignore,

  /// Specifies that the field must be set in order to be valid.
  required: bool,

  /// Specifies that only the values in this list will be considered valid for this field.
  in_: Option<&'static SortedList<Duration>>,

  /// Specifies that the values in this list will be considered NOT valid for this field.
  not_in: Option<&'static SortedList<Duration>>,

  /// Specifies that only this specific value will be considered valid for this field.
  const_: Option<Duration>,

  /// Specifies that the value must be smaller than the indicated amount in order to pass validation.
  lt: Option<Duration>,

  /// Specifies that the value must be equal to or smaller than the indicated amount in order to pass validation.
  lte: Option<Duration>,

  /// Specifies that the value must be greater than the indicated amount in order to pass validation.
  gt: Option<Duration>,

  /// Specifies that the value must be equal to or greater than the indicated amount in order to pass validation.
  gte: Option<Duration>,
}

impl DurationValidator {
  #[must_use]
  pub fn builder() -> DurationValidatorBuilder {
    DurationValidatorBuilder::default()
  }
}

impl<S: State> From<DurationValidatorBuilder<S>> for ProtoOption {
  fn from(value: DurationValidatorBuilder<S>) -> Self {
    value.build().into()
  }
}

#[allow(
  clippy::must_use_candidate,
  clippy::use_self,
  clippy::return_self_not_must_use
)]
impl<S: State> DurationValidatorBuilder<S> {
  pub fn cel(mut self, program: &'static CelProgram) -> DurationValidatorBuilder<S> {
    self.cel.push(program);

    DurationValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
    }
  }

  pub fn ignore_always(self) -> DurationValidatorBuilder<SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    DurationValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: Ignore::Always,
      required: self.required,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
    }
  }

  pub fn required(self) -> DurationValidatorBuilder<SetRequired<S>>
  where
    S::Required: IsUnset,
  {
    DurationValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      required: true,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
    }
  }

  pub fn in_(self, val: &'static SortedList<Duration>) -> DurationValidatorBuilder<SetIn<S>>
  where
    S::In: IsUnset,
  {
    DurationValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      in_: Some(val),
      not_in: self.not_in,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
    }
  }

  pub fn not_in(self, val: &'static SortedList<Duration>) -> DurationValidatorBuilder<SetNotIn<S>>
  where
    S::NotIn: IsUnset,
  {
    DurationValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      in_: self.in_,
      not_in: Some(val),
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
    }
  }

  pub fn const_(self, val: Duration) -> DurationValidatorBuilder<SetConst<S>>
  where
    S::Const: IsUnset,
  {
    DurationValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      in_: self.in_,
      not_in: self.not_in,
      const_: Some(val),
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
    }
  }

  pub fn lt(self, val: Duration) -> DurationValidatorBuilder<SetLt<S>>
  where
    S::Lt: IsUnset,
  {
    DurationValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
      lt: Some(val),
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
    }
  }

  pub fn lte(self, val: Duration) -> DurationValidatorBuilder<SetLte<S>>
  where
    S::Lte: IsUnset,
  {
    DurationValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
      lt: self.lt,
      lte: Some(val),
      gt: self.gt,
      gte: self.gte,
    }
  }

  pub fn gt(self, val: Duration) -> DurationValidatorBuilder<SetGt<S>>
  where
    S::Gt: IsUnset,
  {
    DurationValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: Some(val),
      gte: self.gte,
    }
  }

  pub fn gte(self, val: Duration) -> DurationValidatorBuilder<SetGte<S>>
  where
    S::Gte: IsUnset,
  {
    DurationValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: Some(val),
    }
  }

  pub fn build(self) -> DurationValidator {
    DurationValidator {
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
    }
  }
}
