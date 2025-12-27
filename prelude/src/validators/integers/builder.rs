pub mod state;
use crate::validators::*;
pub use state::*;

#[derive(Clone, Debug, Default)]
pub struct IntValidatorBuilder<Num, S = Empty>
where
  S: State,
  Num: IntWrapper,
{
  _state: PhantomData<S>,
  _wrapper: PhantomData<Num>,

  /// Adds custom validation using one or more [`CelRule`]s to this field.
  cel: Vec<&'static CelProgram>,

  ignore: Ignore,

  /// Specifies that the field must be set in order to be valid.
  required: bool,

  /// Specifies that only this specific value will be considered valid for this field.
  const_: Option<Num::RustType>,

  /// Specifies that this field's value will be valid only if it is smaller than the specified amount
  lt: Option<Num::RustType>,

  /// Specifies that this field's value will be valid only if it is smaller than, or equal to, the specified amount
  lte: Option<Num::RustType>,

  /// Specifies that this field's value will be valid only if it is greater than the specified amount
  gt: Option<Num::RustType>,

  /// Specifies that this field's value will be valid only if it is smaller than, or equal to, the specified amount
  gte: Option<Num::RustType>,

  /// Specifies that only the values in this list will be considered valid for this field.
  in_: Option<&'static SortedList<Num::RustType>>,

  /// Specifies that the values in this list will be considered NOT valid for this field.
  not_in: Option<&'static SortedList<Num::RustType>>,
}

impl<S, N> From<IntValidatorBuilder<N, S>> for ProtoOption
where
  S: State,
  N: IntWrapper,
{
  fn from(value: IntValidatorBuilder<N, S>) -> Self {
    value.build().into()
  }
}

impl<Num, S> IntValidatorBuilder<Num, S>
where
  S: State,
  Num: IntWrapper,
{
  pub fn ignore_always(self) -> IntValidatorBuilder<Num, SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    IntValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      cel: self.cel,
      ignore: Ignore::Always,
      required: self.required,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
      in_: self.in_,
      not_in: self.not_in,
    }
  }

  pub fn ignore_if_zero_value(self) -> IntValidatorBuilder<Num, SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    IntValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      cel: self.cel,
      ignore: Ignore::IfZeroValue,
      required: self.required,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
      in_: self.in_,
      not_in: self.not_in,
    }
  }

  #[allow(clippy::use_self, clippy::return_self_not_must_use)]
  pub fn cel(mut self, program: &'static CelProgram) -> IntValidatorBuilder<Num, S> {
    self.cel.push(program);

    IntValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
      in_: self.in_,
      not_in: self.not_in,
    }
  }

  pub fn required(self) -> IntValidatorBuilder<Num, SetRequired<S>>
  where
    S::Required: IsUnset,
  {
    IntValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      cel: self.cel,
      ignore: self.ignore,
      required: true,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
      in_: self.in_,
      not_in: self.not_in,
    }
  }

  pub fn const_(self, val: Num::RustType) -> IntValidatorBuilder<Num, SetConst<S>>
  where
    S::Const: IsUnset,
  {
    IntValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      const_: Some(val),
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
      in_: self.in_,
      not_in: self.not_in,
    }
  }

  pub fn lt(self, val: Num::RustType) -> IntValidatorBuilder<Num, SetLt<S>>
  where
    S::Lt: IsUnset,
  {
    IntValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      const_: self.const_,
      lt: Some(val),
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
      in_: self.in_,
      not_in: self.not_in,
    }
  }

  pub fn lte(self, val: Num::RustType) -> IntValidatorBuilder<Num, SetLte<S>>
  where
    S::Lte: IsUnset,
  {
    IntValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      const_: self.const_,
      lt: self.lt,
      lte: Some(val),
      gt: self.gt,
      gte: self.gte,
      in_: self.in_,
      not_in: self.not_in,
    }
  }

  pub fn gt(self, val: Num::RustType) -> IntValidatorBuilder<Num, SetGt<S>>
  where
    S::Gt: IsUnset,
  {
    IntValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: Some(val),
      gte: self.gte,
      in_: self.in_,
      not_in: self.not_in,
    }
  }

  pub fn gte(self, val: Num::RustType) -> IntValidatorBuilder<Num, SetGte<S>>
  where
    S::Gte: IsUnset,
  {
    IntValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: Some(val),
      in_: self.in_,
      not_in: self.not_in,
    }
  }

  pub fn not_in(
    self,
    list: &'static SortedList<Num::RustType>,
  ) -> IntValidatorBuilder<Num, SetNotIn<S>>
  where
    S::NotIn: IsUnset,
  {
    IntValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
      not_in: Some(list),
      in_: self.in_,
    }
  }

  pub fn in_(self, list: &'static SortedList<Num::RustType>) -> IntValidatorBuilder<Num, SetIn<S>>
  where
    S::In: IsUnset,
  {
    IntValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
      in_: Some(list),
      not_in: self.not_in,
    }
  }

  pub fn build(self) -> IntValidator<Num> {
    IntValidator {
      cel: self.cel,
      ignore: self.ignore,
      _wrapper: self._wrapper,
      required: self.required,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
      in_: self.in_,
      not_in: self.not_in,
    }
  }
}
