pub mod state;
use crate::validators::*;
pub use state::*;

use proto_types::{Duration, Timestamp};

#[derive(Clone, Debug, Default)]
pub struct TimestampValidatorBuilder<S: State = Empty> {
  _state: PhantomData<S>,

  /// Adds custom validation using one or more [`CelRule`]s to this field.
  cel: Vec<&'static CelProgram>,

  ignore: Ignore,

  /// Specifies that this field's value will be valid only if it in the past.
  lt_now: bool,

  /// Specifies that this field's value will be valid only if it in the future.
  gt_now: bool,

  /// Specifies that the field must be set in order to be valid.
  required: bool,

  /// Specifies that only this specific value will be considered valid for this field.
  const_: Option<Timestamp>,

  /// Specifies that this field's value will be valid only if it is smaller than the specified amount.
  lt: Option<Timestamp>,

  /// Specifies that this field's value will be valid only if it is smaller than, or equal to, the specified amount.
  lte: Option<Timestamp>,

  /// Specifies that this field's value will be valid only if it is greater than the specified amount.
  gt: Option<Timestamp>,

  /// Specifies that this field's value will be valid only if it is greater than, or equal to, the specified amount.
  gte: Option<Timestamp>,

  /// Specifies that this field's value will be valid only if it is within the specified Duration (either in the past or future) from the moment when it's being validated.
  within: Option<Duration>,

  now_tolerance: Duration,
}

impl TimestampValidator {
  #[must_use]
  pub fn builder() -> TimestampValidatorBuilder {
    TimestampValidatorBuilder::default()
  }
}

impl<S: State> From<TimestampValidatorBuilder<S>> for ProtoOption {
  fn from(value: TimestampValidatorBuilder<S>) -> Self {
    value.build().into()
  }
}

#[allow(
  clippy::must_use_candidate,
  clippy::use_self,
  clippy::return_self_not_must_use
)]
impl<S: State> TimestampValidatorBuilder<S> {
  pub fn cel(mut self, program: &'static CelProgram) -> TimestampValidatorBuilder<S> {
    self.cel.push(program);

    TimestampValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      lt_now: self.lt_now,
      gt_now: self.gt_now,
      required: self.required,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
      within: self.within,
      now_tolerance: self.now_tolerance,
    }
  }

  pub fn ignore_always(self) -> TimestampValidatorBuilder<SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    TimestampValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: Ignore::Always,
      lt_now: self.lt_now,
      gt_now: self.gt_now,
      required: self.required,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
      within: self.within,
      now_tolerance: self.now_tolerance,
    }
  }

  pub fn lt_now(self) -> TimestampValidatorBuilder<SetLtNow<S>>
  where
    S::LtNow: IsUnset,
  {
    TimestampValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      lt_now: true,
      gt_now: self.gt_now,
      required: self.required,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
      within: self.within,
      now_tolerance: self.now_tolerance,
    }
  }

  pub fn gt_now(self) -> TimestampValidatorBuilder<SetGtNow<S>>
  where
    S::GtNow: IsUnset,
  {
    TimestampValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      lt_now: self.lt_now,
      gt_now: true,
      required: self.required,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
      within: self.within,
      now_tolerance: self.now_tolerance,
    }
  }

  pub fn required(self) -> TimestampValidatorBuilder<SetRequired<S>>
  where
    S::Required: IsUnset,
  {
    TimestampValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      lt_now: self.lt_now,
      gt_now: self.gt_now,
      required: true,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
      within: self.within,
      now_tolerance: self.now_tolerance,
    }
  }

  pub fn const_(self, val: Timestamp) -> TimestampValidatorBuilder<SetConst<S>>
  where
    S::Const: IsUnset,
  {
    TimestampValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      lt_now: self.lt_now,
      gt_now: self.gt_now,
      required: self.required,
      const_: Some(val),
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
      within: self.within,
      now_tolerance: self.now_tolerance,
    }
  }

  pub fn lt(self, val: Timestamp) -> TimestampValidatorBuilder<SetLt<S>>
  where
    S::Lt: IsUnset,
  {
    TimestampValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      lt_now: self.lt_now,
      gt_now: self.gt_now,
      required: self.required,
      const_: self.const_,
      lt: Some(val),
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
      within: self.within,
      now_tolerance: self.now_tolerance,
    }
  }

  pub fn lte(self, val: Timestamp) -> TimestampValidatorBuilder<SetLte<S>>
  where
    S::Lte: IsUnset,
  {
    TimestampValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      lt_now: self.lt_now,
      gt_now: self.gt_now,
      required: self.required,
      const_: self.const_,
      lt: self.lt,
      lte: Some(val),
      gt: self.gt,
      gte: self.gte,
      within: self.within,
      now_tolerance: self.now_tolerance,
    }
  }

  pub fn gt(self, val: Timestamp) -> TimestampValidatorBuilder<SetGt<S>>
  where
    S::Gt: IsUnset,
  {
    TimestampValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      lt_now: self.lt_now,
      gt_now: self.gt_now,
      required: self.required,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: Some(val),
      gte: self.gte,
      within: self.within,
      now_tolerance: self.now_tolerance,
    }
  }

  pub fn gte(self, val: Timestamp) -> TimestampValidatorBuilder<SetGte<S>>
  where
    S::Gte: IsUnset,
  {
    TimestampValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      lt_now: self.lt_now,
      gt_now: self.gt_now,
      required: self.required,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: Some(val),
      within: self.within,
      now_tolerance: self.now_tolerance,
    }
  }

  pub fn within(self, val: Duration) -> TimestampValidatorBuilder<SetWithin<S>>
  where
    S::Within: IsUnset,
  {
    TimestampValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      lt_now: self.lt_now,
      gt_now: self.gt_now,
      required: self.required,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
      within: Some(val),
      now_tolerance: self.now_tolerance,
    }
  }

  pub fn now_tolerance(self, val: Duration) -> TimestampValidatorBuilder<SetNowTolerance<S>>
  where
    S::NowTolerance: IsUnset,
  {
    TimestampValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      lt_now: self.lt_now,
      gt_now: self.gt_now,
      required: self.required,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
      within: self.within,
      now_tolerance: val,
    }
  }

  pub fn build(self) -> TimestampValidator {
    TimestampValidator {
      cel: self.cel,
      ignore: self.ignore,
      lt_now: self.lt_now,
      gt_now: self.gt_now,
      required: self.required,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
      within: self.within,
      now_tolerance: self.now_tolerance,
    }
  }
}
