pub mod state;
use crate::validators::*;
pub use state::*;

#[derive(Debug, Clone, Default)]
pub struct FloatValidatorBuilder<Num, S = Empty>
where
  S: State,
  Num: FloatWrapper,
{
  _wrapper: PhantomData<Num>,
  _state: PhantomData<S>,

  /// Adds custom validation using one or more [`CelRule`]s to this field.
  cel: Vec<&'static CelProgram>,

  ignore: Ignore,

  /// Specifies that the field must be set in order to be valid.
  required: bool,

  /// The absolute tolerance to use for equality operations
  abs_tolerance: Num::RustType,

  /// The relative tolerance to use for equality operations, scaled to the precision of the number being validated
  rel_tolerance: Num::RustType,

  /// Specifies that this field must be finite (i.e. it can't represent Infinity or NaN)
  finite: bool,

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
  in_: Option<&'static SortedList<OrderedFloat<Num::RustType>>>,

  /// Specifies that the values in this list will be considered NOT valid for this field.
  not_in: Option<&'static SortedList<OrderedFloat<Num::RustType>>>,
}

impl<Num, S> FloatValidatorBuilder<Num, S>
where
  S: State,
  Num: FloatWrapper,
{
  pub fn ignore_always(self) -> FloatValidatorBuilder<Num, SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    FloatValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      cel: self.cel,
      ignore: Ignore::Always,
      required: self.required,
      abs_tolerance: self.abs_tolerance,
      rel_tolerance: self.rel_tolerance,
      finite: self.finite,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
      in_: self.in_,
      not_in: self.not_in,
    }
  }

  pub fn ignore_if_zero_value(self) -> FloatValidatorBuilder<Num, SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    FloatValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      cel: self.cel,
      ignore: Ignore::IfZeroValue,
      required: self.required,
      abs_tolerance: self.abs_tolerance,
      rel_tolerance: self.rel_tolerance,
      finite: self.finite,
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
  pub fn cel(mut self, program: &'static CelProgram) -> FloatValidatorBuilder<Num, S> {
    self.cel.push(program);

    FloatValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      abs_tolerance: self.abs_tolerance,
      rel_tolerance: self.rel_tolerance,
      finite: self.finite,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
      in_: self.in_,
      not_in: self.not_in,
    }
  }

  pub fn required(self) -> FloatValidatorBuilder<Num, SetRequired<S>>
  where
    S::Required: IsUnset,
  {
    FloatValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      cel: self.cel,
      ignore: self.ignore,
      required: true,
      abs_tolerance: self.abs_tolerance,
      rel_tolerance: self.rel_tolerance,
      finite: self.finite,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
      in_: self.in_,
      not_in: self.not_in,
    }
  }

  pub fn abs_tolerance(self, val: Num::RustType) -> FloatValidatorBuilder<Num, SetAbsTolerance<S>>
  where
    S::AbsTolerance: IsUnset,
  {
    FloatValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      abs_tolerance: val,
      rel_tolerance: self.rel_tolerance,
      finite: self.finite,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
      in_: self.in_,
      not_in: self.not_in,
    }
  }

  pub fn rel_tolerance(self, val: Num::RustType) -> FloatValidatorBuilder<Num, SetRelTolerance<S>>
  where
    S::RelTolerance: IsUnset,
  {
    FloatValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      abs_tolerance: self.abs_tolerance,
      rel_tolerance: val,
      finite: self.finite,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
      in_: self.in_,
      not_in: self.not_in,
    }
  }

  pub fn finite(self) -> FloatValidatorBuilder<Num, SetFinite<S>>
  where
    S::Finite: IsUnset,
  {
    FloatValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      abs_tolerance: self.abs_tolerance,
      rel_tolerance: self.rel_tolerance,
      finite: true,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
      in_: self.in_,
      not_in: self.not_in,
    }
  }

  pub fn const_(self, val: Num::RustType) -> FloatValidatorBuilder<Num, SetConst<S>>
  where
    S::Const: IsUnset,
  {
    FloatValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      abs_tolerance: self.abs_tolerance,
      rel_tolerance: self.rel_tolerance,
      finite: self.finite,
      const_: Some(val),
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
      in_: self.in_,
      not_in: self.not_in,
    }
  }

  pub fn lt(self, val: Num::RustType) -> FloatValidatorBuilder<Num, SetLt<S>>
  where
    S::Lt: IsUnset,
  {
    FloatValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      abs_tolerance: self.abs_tolerance,
      rel_tolerance: self.rel_tolerance,
      finite: self.finite,
      const_: self.const_,
      lt: Some(val),
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
      in_: self.in_,
      not_in: self.not_in,
    }
  }

  pub fn lte(self, val: Num::RustType) -> FloatValidatorBuilder<Num, SetLte<S>>
  where
    S::Lte: IsUnset,
  {
    FloatValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      abs_tolerance: self.abs_tolerance,
      rel_tolerance: self.rel_tolerance,
      finite: self.finite,
      const_: self.const_,
      lt: self.lt,
      lte: Some(val),
      gt: self.gt,
      gte: self.gte,
      in_: self.in_,
      not_in: self.not_in,
    }
  }

  pub fn gt(self, val: Num::RustType) -> FloatValidatorBuilder<Num, SetGt<S>>
  where
    S::Gt: IsUnset,
  {
    FloatValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      abs_tolerance: self.abs_tolerance,
      rel_tolerance: self.rel_tolerance,
      finite: self.finite,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: Some(val),
      gte: self.gte,
      in_: self.in_,
      not_in: self.not_in,
    }
  }

  pub fn gte(self, val: Num::RustType) -> FloatValidatorBuilder<Num, SetGte<S>>
  where
    S::Gte: IsUnset,
  {
    FloatValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      abs_tolerance: self.abs_tolerance,
      rel_tolerance: self.rel_tolerance,
      finite: self.finite,
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
    list: &'static SortedList<OrderedFloat<Num::RustType>>,
  ) -> FloatValidatorBuilder<Num, SetNotIn<S>>
  where
    S::NotIn: IsUnset,
  {
    FloatValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      abs_tolerance: self.abs_tolerance,
      rel_tolerance: self.rel_tolerance,
      finite: self.finite,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
      not_in: Some(list),
      in_: self.in_,
    }
  }

  pub fn in_(
    self,
    list: &'static SortedList<OrderedFloat<Num::RustType>>,
  ) -> FloatValidatorBuilder<Num, SetIn<S>>
  where
    S::In: IsUnset,
  {
    FloatValidatorBuilder {
      _state: PhantomData,
      _wrapper: self._wrapper,
      cel: self.cel,
      ignore: self.ignore,
      required: self.required,
      abs_tolerance: self.abs_tolerance,
      rel_tolerance: self.rel_tolerance,
      finite: self.finite,
      const_: self.const_,
      lt: self.lt,
      lte: self.lte,
      gt: self.gt,
      gte: self.gte,
      in_: Some(list),
      not_in: self.not_in,
    }
  }

  pub fn build(self) -> FloatValidator<Num> {
    FloatValidator {
      cel: self.cel,
      ignore: self.ignore,
      _wrapper: self._wrapper,
      required: self.required,
      abs_tolerance: self.abs_tolerance,
      rel_tolerance: self.rel_tolerance,
      finite: self.finite,
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
