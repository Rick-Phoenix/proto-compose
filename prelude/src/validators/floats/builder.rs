#[doc(hidden)]
pub mod state;
use crate::validators::*;
pub(crate) use state::*;

#[derive(Debug, Clone)]
pub struct FloatValidatorBuilder<Num, S = Empty>
where
  S: State,
  Num: FloatWrapper,
{
  _wrapper: PhantomData<Num>,
  _state: PhantomData<S>,

  data: FloatValidator<Num>,
}

impl<Num, S> Default for FloatValidatorBuilder<Num, S>
where
  S: State,
  Num: FloatWrapper,
{
  #[inline]
  fn default() -> Self {
    Self {
      _wrapper: PhantomData,
      _state: PhantomData,
      data: FloatValidator::default(),
    }
  }
}

impl<Num, S> FloatValidatorBuilder<Num, S>
where
  S: State,
  Num: FloatWrapper,
{
  #[inline]
  pub fn with_error_messages(
    mut self,
    error_messages: impl IntoIterator<Item = (Num::ViolationEnum, impl Into<FixedStr>)>,
  ) -> FloatValidatorBuilder<Num, SetErrorMessages<S>>
  where
    S::ErrorMessages: IsUnset,
  {
    let map: BTreeMap<Num::ViolationEnum, FixedStr> = error_messages
      .into_iter()
      .map(|(v, m)| (v, m.into()))
      .collect();
    self.data.error_messages = Some(Box::new(map));

    FloatValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      data: self.data,
    }
  }

  #[inline]
  pub fn ignore_always(mut self) -> FloatValidatorBuilder<Num, SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    self.data.ignore = Ignore::Always;

    FloatValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      data: self.data,
    }
  }

  #[inline]
  pub fn ignore_if_zero_value(mut self) -> FloatValidatorBuilder<Num, SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    self.data.ignore = Ignore::IfZeroValue;

    FloatValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      data: self.data,
    }
  }

  #[inline]
  #[allow(clippy::use_self, clippy::return_self_not_must_use)]
  pub fn cel(mut self, program: CelProgram) -> FloatValidatorBuilder<Num, S> {
    self.data.cel.push(program);

    FloatValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      data: self.data,
    }
  }

  #[inline]
  pub fn required(mut self) -> FloatValidatorBuilder<Num, SetRequired<S>>
  where
    S::Required: IsUnset,
  {
    self.data.required = true;

    FloatValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      data: self.data,
    }
  }

  #[inline]
  pub fn abs_tolerance(mut self, val: Num) -> FloatValidatorBuilder<Num, SetAbsTolerance<S>>
  where
    S::AbsTolerance: IsUnset,
  {
    self.data.abs_tolerance = val;

    FloatValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      data: self.data,
    }
  }

  #[inline]
  pub fn rel_tolerance(mut self, val: Num) -> FloatValidatorBuilder<Num, SetRelTolerance<S>>
  where
    S::RelTolerance: IsUnset,
  {
    self.data.rel_tolerance = val;

    FloatValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      data: self.data,
    }
  }

  #[inline]
  pub fn finite(mut self) -> FloatValidatorBuilder<Num, SetFinite<S>>
  where
    S::Finite: IsUnset,
  {
    self.data.finite = true;

    FloatValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      data: self.data,
    }
  }

  #[inline]
  pub fn const_(mut self, val: Num) -> FloatValidatorBuilder<Num, SetConst<S>>
  where
    S::Const: IsUnset,
  {
    self.data.const_ = Some(val);

    FloatValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      data: self.data,
    }
  }

  #[inline]
  pub fn lt(mut self, val: Num) -> FloatValidatorBuilder<Num, SetLt<S>>
  where
    S::Lt: IsUnset,
  {
    self.data.lt = Some(val);

    FloatValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      data: self.data,
    }
  }

  #[inline]
  pub fn lte(mut self, val: Num) -> FloatValidatorBuilder<Num, SetLte<S>>
  where
    S::Lte: IsUnset,
  {
    self.data.lte = Some(val);

    FloatValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      data: self.data,
    }
  }

  #[inline]
  pub fn gt(mut self, val: Num) -> FloatValidatorBuilder<Num, SetGt<S>>
  where
    S::Gt: IsUnset,
  {
    self.data.gt = Some(val);

    FloatValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      data: self.data,
    }
  }

  #[inline]
  pub fn gte(mut self, val: Num) -> FloatValidatorBuilder<Num, SetGte<S>>
  where
    S::Gte: IsUnset,
  {
    self.data.gte = Some(val);

    FloatValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      data: self.data,
    }
  }

  #[inline]
  pub fn not_in(
    mut self,
    list: impl IntoSortedList<OrderedFloat<Num>>,
  ) -> FloatValidatorBuilder<Num, SetNotIn<S>>
  where
    S::NotIn: IsUnset,
  {
    self.data.not_in = Some(list.into_sorted_list());

    FloatValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      data: self.data,
    }
  }

  #[inline]
  pub fn in_(
    mut self,
    list: impl IntoSortedList<OrderedFloat<Num>>,
  ) -> FloatValidatorBuilder<Num, SetIn<S>>
  where
    S::In: IsUnset,
  {
    self.data.in_ = Some(list.into_sorted_list());

    FloatValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      data: self.data,
    }
  }

  #[inline]
  pub fn build(self) -> FloatValidator<Num> {
    self.data
  }
}
