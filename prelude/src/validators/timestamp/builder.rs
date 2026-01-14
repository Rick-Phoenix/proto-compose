#[doc(hidden)]
pub mod state;
use crate::validators::*;
pub(crate) use state::*;

use proto_types::{Duration, Timestamp};

#[derive(Clone, Debug)]
pub struct TimestampValidatorBuilder<S: State = Empty> {
  _state: PhantomData<S>,

  data: TimestampValidator,
}

impl_validator!(TimestampValidator, Timestamp);

impl<S: State> Default for TimestampValidatorBuilder<S> {
  #[inline]
  fn default() -> Self {
    Self {
      _state: PhantomData,
      data: TimestampValidator::default(),
    }
  }
}

impl TimestampValidator {
  #[must_use]
  #[inline]
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
  #[inline]
  pub fn cel(mut self, program: CelProgram) -> TimestampValidatorBuilder<S> {
    self.data.cel.push(program);

    TimestampValidatorBuilder {
      _state: PhantomData,
      ..self
    }
  }

  #[inline]
  pub fn ignore_always(mut self) -> TimestampValidatorBuilder<SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    self.data.ignore = Ignore::Always;

    TimestampValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[cfg(all(feature = "chrono", any(feature = "std", feature = "chrono-wasm")))]
  #[inline]
  pub fn lt_now(mut self) -> TimestampValidatorBuilder<SetLtNow<S>>
  where
    S::LtNow: IsUnset,
  {
    self.data.lt_now = true;

    TimestampValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[cfg(all(feature = "chrono", any(feature = "std", feature = "chrono-wasm")))]
  #[inline]
  pub fn gt_now(mut self) -> TimestampValidatorBuilder<SetGtNow<S>>
  where
    S::GtNow: IsUnset,
  {
    self.data.gt_now = true;

    TimestampValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn required(mut self) -> TimestampValidatorBuilder<SetRequired<S>>
  where
    S::Required: IsUnset,
  {
    self.data.required = true;

    TimestampValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn const_(mut self, val: Timestamp) -> TimestampValidatorBuilder<SetConst<S>>
  where
    S::Const: IsUnset,
  {
    self.data.const_ = Some(val);

    TimestampValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn lt(mut self, val: Timestamp) -> TimestampValidatorBuilder<SetLt<S>>
  where
    S::Lt: IsUnset,
  {
    self.data.lt = Some(val);

    TimestampValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn lte(mut self, val: Timestamp) -> TimestampValidatorBuilder<SetLte<S>>
  where
    S::Lte: IsUnset,
  {
    self.data.lte = Some(val);

    TimestampValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn gt(mut self, val: Timestamp) -> TimestampValidatorBuilder<SetGt<S>>
  where
    S::Gt: IsUnset,
  {
    self.data.gt = Some(val);

    TimestampValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn gte(mut self, val: Timestamp) -> TimestampValidatorBuilder<SetGte<S>>
  where
    S::Gte: IsUnset,
  {
    self.data.gte = Some(val);

    TimestampValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[cfg(all(feature = "chrono", any(feature = "std", feature = "chrono-wasm")))]
  #[inline]
  pub fn within(mut self, val: Duration) -> TimestampValidatorBuilder<SetWithin<S>>
  where
    S::Within: IsUnset,
  {
    self.data.within = Some(val);

    TimestampValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[cfg(all(feature = "chrono", any(feature = "std", feature = "chrono-wasm")))]
  #[inline]
  pub fn now_tolerance(mut self, val: Duration) -> TimestampValidatorBuilder<SetNowTolerance<S>>
  where
    S::NowTolerance: IsUnset,
  {
    self.data.now_tolerance = val;

    TimestampValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn build(self) -> TimestampValidator {
    self.data
  }
}
