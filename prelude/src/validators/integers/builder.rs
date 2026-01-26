#[doc(hidden)]
pub mod state;
use crate::validators::*;
pub(crate) use state::*;

#[derive(Clone, Debug)]
pub struct IntValidatorBuilder<Num, S = Empty>
where
  S: State,
  Num: IntWrapper,
{
  _state: PhantomData<S>,
  _wrapper: PhantomData<Num>,

  data: IntValidator<Num>,
}

impl<Num, S> Default for IntValidatorBuilder<Num, S>
where
  S: State,
  Num: IntWrapper,
{
  #[inline]
  fn default() -> Self {
    Self {
      _state: PhantomData,
      _wrapper: PhantomData,
      data: IntValidator::default(),
    }
  }
}

impl<S, N> From<IntValidatorBuilder<N, S>> for ProtoOption
where
  S: State,
  N: IntWrapper,
{
  #[inline(never)]
  #[cold]
  fn from(value: IntValidatorBuilder<N, S>) -> Self {
    value.build().into()
  }
}

impl<Num, S> IntValidatorBuilder<Num, S>
where
  S: State,
  Num: IntWrapper,
{
  #[inline]
  pub fn with_error_messages(
    mut self,
    error_messages: impl IntoIterator<Item = (Num::ViolationEnum, impl Into<FixedStr>)>,
  ) -> IntValidatorBuilder<Num, SetErrorMessages<S>>
  where
    S::ErrorMessages: IsUnset,
  {
    let map: BTreeMap<Num::ViolationEnum, FixedStr> = error_messages
      .into_iter()
      .map(|(v, m)| (v, m.into()))
      .collect();
    self.data.error_messages = Some(Box::new(map));

    IntValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      data: self.data,
    }
  }

  #[inline]
  pub fn ignore_always(mut self) -> IntValidatorBuilder<Num, SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    self.data.ignore = Ignore::Always;

    IntValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      data: self.data,
    }
  }

  #[inline]
  pub fn ignore_if_zero_value(mut self) -> IntValidatorBuilder<Num, SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    self.data.ignore = Ignore::IfZeroValue;

    IntValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      data: self.data,
    }
  }

  #[inline]
  #[allow(clippy::use_self, clippy::return_self_not_must_use)]
  pub fn cel(mut self, program: CelProgram) -> IntValidatorBuilder<Num, S> {
    self.data.cel.push(program);

    IntValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      data: self.data,
    }
  }

  #[inline]
  pub fn required(mut self) -> IntValidatorBuilder<Num, SetRequired<S>>
  where
    S::Required: IsUnset,
  {
    self.data.required = true;

    IntValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      data: self.data,
    }
  }

  #[inline]
  pub fn const_(mut self, val: Num::RustType) -> IntValidatorBuilder<Num, SetConst<S>>
  where
    S::Const: IsUnset,
  {
    self.data.const_ = Some(val);

    IntValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      data: self.data,
    }
  }

  #[inline]
  pub fn lt(mut self, val: Num::RustType) -> IntValidatorBuilder<Num, SetLt<S>>
  where
    S::Lt: IsUnset,
  {
    self.data.lt = Some(val);

    IntValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      data: self.data,
    }
  }

  #[inline]
  pub fn lte(mut self, val: Num::RustType) -> IntValidatorBuilder<Num, SetLte<S>>
  where
    S::Lte: IsUnset,
  {
    self.data.lte = Some(val);

    IntValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      data: self.data,
    }
  }

  #[inline]
  pub fn gt(mut self, val: Num::RustType) -> IntValidatorBuilder<Num, SetGt<S>>
  where
    S::Gt: IsUnset,
  {
    self.data.gt = Some(val);

    IntValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      data: self.data,
    }
  }

  #[inline]
  pub fn gte(mut self, val: Num::RustType) -> IntValidatorBuilder<Num, SetGte<S>>
  where
    S::Gte: IsUnset,
  {
    self.data.gte = Some(val);

    IntValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      data: self.data,
    }
  }

  #[inline]
  pub fn not_in(
    mut self,
    list: impl IntoSortedList<Num::RustType>,
  ) -> IntValidatorBuilder<Num, SetNotIn<S>>
  where
    S::NotIn: IsUnset,
  {
    self.data.not_in = Some(list.into_sorted_list());

    IntValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      data: self.data,
    }
  }

  #[inline]
  pub fn in_(
    mut self,
    list: impl IntoSortedList<Num::RustType>,
  ) -> IntValidatorBuilder<Num, SetIn<S>>
  where
    S::In: IsUnset,
  {
    self.data.in_ = Some(list.into_sorted_list());

    IntValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      data: self.data,
    }
  }

  #[inline]
  pub fn build(self) -> IntValidator<Num> {
    self.data
  }
}
