mod well_known;
pub use well_known::*;
pub mod state;
use crate::validators::*;
pub use state::*;

#[cfg(feature = "regex")]
use regex::Regex;

#[derive(Clone, Debug, Default)]
pub struct StringValidatorBuilder<S: State = Empty> {
  _state: PhantomData<S>,
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  cel: Vec<&'static CelProgram>,

  pub(crate) well_known: Option<WellKnownStrings>,

  ignore: Ignore,

  /// Specifies that the field must be set in order to be valid.
  required: bool,

  /// Specifies that the given string field must be of this exact length.
  len: Option<usize>,

  /// Specifies that the given string field must have a length that is equal to or higher than the given value.
  min_len: Option<usize>,

  /// Specifies that the given string field must have a length that is equal to or lower than the given value.
  max_len: Option<usize>,

  /// Specifies the exact byte length that this field's value must have in order to be considered valid.
  len_bytes: Option<usize>,

  /// Specifies the minimum byte length for this field's value to be considered valid.
  min_bytes: Option<usize>,

  /// Specifies the minimum byte length for this field's value to be considered valid.
  max_bytes: Option<usize>,

  #[cfg(feature = "regex")]
  /// Specifies a regex pattern that this field's value should match in order to be considered valid.
  pattern: Option<&'static Regex>,

  /// Specifies the prefix that this field's value should contain in order to be considered valid.
  prefix: Option<Arc<str>>,

  /// Specifies the suffix that this field's value should contain in order to be considered valid.
  suffix: Option<Arc<str>>,

  /// Specifies a substring that this field's value should contain in order to be considered valid.
  contains: Option<Arc<str>>,

  /// Specifies a substring that this field's value must not contain in order to be considered valid.
  not_contains: Option<Arc<str>>,

  /// Specifies that only the values in this list will be considered valid for this field.
  in_: Option<&'static SortedList<&'static str>>,

  /// Specifies that the values in this list will be considered NOT valid for this field.
  not_in: Option<&'static SortedList<&'static str>>,

  /// Specifies that only this specific value will be considered valid for this field.
  const_: Option<Arc<str>>,
}

impl StringValidator {
  #[must_use]
  pub fn builder() -> StringValidatorBuilder {
    StringValidatorBuilder::default()
  }
}

impl<S: State> From<StringValidatorBuilder<S>> for ProtoOption {
  fn from(value: StringValidatorBuilder<S>) -> Self {
    value.build().into()
  }
}

#[allow(
  clippy::must_use_candidate,
  clippy::use_self,
  clippy::return_self_not_must_use
)]
impl<S: State> StringValidatorBuilder<S> {
  pub fn cel(mut self, program: &'static CelProgram) -> StringValidatorBuilder<S> {
    self.cel.push(program);

    StringValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      well_known: self.well_known,
      ignore: self.ignore,
      required: self.required,
      len: self.len,
      min_len: self.min_len,
      max_len: self.max_len,
      len_bytes: self.len_bytes,
      min_bytes: self.min_bytes,
      max_bytes: self.max_bytes,
      #[cfg(feature = "regex")]
      pattern: self.pattern,
      prefix: self.prefix,
      suffix: self.suffix,
      contains: self.contains,
      not_contains: self.not_contains,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn ignore_always(self) -> StringValidatorBuilder<SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    StringValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      well_known: self.well_known,
      ignore: Ignore::Always,
      required: self.required,
      len: self.len,
      min_len: self.min_len,
      max_len: self.max_len,
      len_bytes: self.len_bytes,
      min_bytes: self.min_bytes,
      max_bytes: self.max_bytes,
      #[cfg(feature = "regex")]
      pattern: self.pattern,
      prefix: self.prefix,
      suffix: self.suffix,
      contains: self.contains,
      not_contains: self.not_contains,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn ignore_if_zero_value(self) -> StringValidatorBuilder<SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    StringValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      well_known: self.well_known,
      ignore: Ignore::IfZeroValue,
      required: self.required,
      len: self.len,
      min_len: self.min_len,
      max_len: self.max_len,
      len_bytes: self.len_bytes,
      min_bytes: self.min_bytes,
      max_bytes: self.max_bytes,
      #[cfg(feature = "regex")]
      pattern: self.pattern,
      prefix: self.prefix,
      suffix: self.suffix,
      contains: self.contains,
      not_contains: self.not_contains,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn required(self) -> StringValidatorBuilder<SetRequired<S>>
  where
    S::Required: IsUnset,
  {
    StringValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      well_known: self.well_known,
      ignore: self.ignore,
      required: true,
      len: self.len,
      min_len: self.min_len,
      max_len: self.max_len,
      len_bytes: self.len_bytes,
      min_bytes: self.min_bytes,
      max_bytes: self.max_bytes,
      #[cfg(feature = "regex")]
      pattern: self.pattern,
      prefix: self.prefix,
      suffix: self.suffix,
      contains: self.contains,
      not_contains: self.not_contains,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn len(self, val: usize) -> StringValidatorBuilder<SetLen<S>>
  where
    S::Len: IsUnset,
  {
    StringValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      well_known: self.well_known,
      ignore: self.ignore,
      required: self.required,
      len: Some(val),
      min_len: self.min_len,
      max_len: self.max_len,
      len_bytes: self.len_bytes,
      min_bytes: self.min_bytes,
      max_bytes: self.max_bytes,
      #[cfg(feature = "regex")]
      pattern: self.pattern,
      prefix: self.prefix,
      suffix: self.suffix,
      contains: self.contains,
      not_contains: self.not_contains,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn min_len(self, val: usize) -> StringValidatorBuilder<SetMinLen<S>>
  where
    S::MinLen: IsUnset,
  {
    StringValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      well_known: self.well_known,
      ignore: self.ignore,
      required: self.required,
      len: self.len,
      min_len: Some(val),
      max_len: self.max_len,
      len_bytes: self.len_bytes,
      min_bytes: self.min_bytes,
      max_bytes: self.max_bytes,
      #[cfg(feature = "regex")]
      pattern: self.pattern,
      prefix: self.prefix,
      suffix: self.suffix,
      contains: self.contains,
      not_contains: self.not_contains,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn max_len(self, val: usize) -> StringValidatorBuilder<SetMaxLen<S>>
  where
    S::MaxLen: IsUnset,
  {
    StringValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      well_known: self.well_known,
      ignore: self.ignore,
      required: self.required,
      len: self.len,
      min_len: self.min_len,
      max_len: Some(val),
      len_bytes: self.len_bytes,
      min_bytes: self.min_bytes,
      max_bytes: self.max_bytes,
      #[cfg(feature = "regex")]
      pattern: self.pattern,
      prefix: self.prefix,
      suffix: self.suffix,
      contains: self.contains,
      not_contains: self.not_contains,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn len_bytes(self, val: usize) -> StringValidatorBuilder<SetLenBytes<S>>
  where
    S::LenBytes: IsUnset,
  {
    StringValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      well_known: self.well_known,
      ignore: self.ignore,
      required: self.required,
      len: self.len,
      min_len: self.min_len,
      max_len: self.max_len,
      len_bytes: Some(val),
      min_bytes: self.min_bytes,
      max_bytes: self.max_bytes,
      #[cfg(feature = "regex")]
      pattern: self.pattern,
      prefix: self.prefix,
      suffix: self.suffix,
      contains: self.contains,
      not_contains: self.not_contains,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn min_bytes(self, val: usize) -> StringValidatorBuilder<SetMinBytes<S>>
  where
    S::MinBytes: IsUnset,
  {
    StringValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      well_known: self.well_known,
      ignore: self.ignore,
      required: self.required,
      len: self.len,
      min_len: self.min_len,
      max_len: self.max_len,
      len_bytes: self.len_bytes,
      min_bytes: Some(val),
      max_bytes: self.max_bytes,
      #[cfg(feature = "regex")]
      pattern: self.pattern,
      prefix: self.prefix,
      suffix: self.suffix,
      contains: self.contains,
      not_contains: self.not_contains,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn max_bytes(self, val: usize) -> StringValidatorBuilder<SetMaxBytes<S>>
  where
    S::MaxBytes: IsUnset,
  {
    StringValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      well_known: self.well_known,
      ignore: self.ignore,
      required: self.required,
      len: self.len,
      min_len: self.min_len,
      max_len: self.max_len,
      len_bytes: self.len_bytes,
      min_bytes: self.min_bytes,
      max_bytes: Some(val),
      #[cfg(feature = "regex")]
      pattern: self.pattern,
      prefix: self.prefix,
      suffix: self.suffix,
      contains: self.contains,
      not_contains: self.not_contains,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  #[cfg(feature = "regex")]
  pub fn pattern(self, val: &'static Regex) -> StringValidatorBuilder<SetPattern<S>>
  where
    S::Pattern: IsUnset,
  {
    StringValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      well_known: self.well_known,
      ignore: self.ignore,
      required: self.required,
      len: self.len,
      min_len: self.min_len,
      max_len: self.max_len,
      len_bytes: self.len_bytes,
      min_bytes: self.min_bytes,
      max_bytes: self.max_bytes,
      pattern: Some(val),
      prefix: self.prefix,
      suffix: self.suffix,
      contains: self.contains,
      not_contains: self.not_contains,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn prefix<T: Into<Arc<str>>>(self, val: T) -> StringValidatorBuilder<SetPrefix<S>>
  where
    S::Prefix: IsUnset,
  {
    StringValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      well_known: self.well_known,
      ignore: self.ignore,
      required: self.required,
      len: self.len,
      min_len: self.min_len,
      max_len: self.max_len,
      len_bytes: self.len_bytes,
      min_bytes: self.min_bytes,
      max_bytes: self.max_bytes,
      #[cfg(feature = "regex")]
      pattern: self.pattern,
      prefix: Some(val.into()),
      suffix: self.suffix,
      contains: self.contains,
      not_contains: self.not_contains,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn suffix<T: Into<Arc<str>>>(self, val: T) -> StringValidatorBuilder<SetSuffix<S>>
  where
    S::Suffix: IsUnset,
  {
    StringValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      well_known: self.well_known,
      ignore: self.ignore,
      required: self.required,
      len: self.len,
      min_len: self.min_len,
      max_len: self.max_len,
      len_bytes: self.len_bytes,
      min_bytes: self.min_bytes,
      max_bytes: self.max_bytes,
      #[cfg(feature = "regex")]
      pattern: self.pattern,
      prefix: self.prefix,
      suffix: Some(val.into()),
      contains: self.contains,
      not_contains: self.not_contains,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn contains<T: Into<Arc<str>>>(self, val: T) -> StringValidatorBuilder<SetContains<S>>
  where
    S::Contains: IsUnset,
  {
    StringValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      well_known: self.well_known,
      ignore: self.ignore,
      required: self.required,
      len: self.len,
      min_len: self.min_len,
      max_len: self.max_len,
      len_bytes: self.len_bytes,
      min_bytes: self.min_bytes,
      max_bytes: self.max_bytes,
      #[cfg(feature = "regex")]
      pattern: self.pattern,
      prefix: self.prefix,
      suffix: self.suffix,
      contains: Some(val.into()),
      not_contains: self.not_contains,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn not_contains<T: Into<Arc<str>>>(self, val: T) -> StringValidatorBuilder<SetNotContains<S>>
  where
    S::NotContains: IsUnset,
  {
    StringValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      well_known: self.well_known,
      ignore: self.ignore,
      required: self.required,
      len: self.len,
      min_len: self.min_len,
      max_len: self.max_len,
      len_bytes: self.len_bytes,
      min_bytes: self.min_bytes,
      max_bytes: self.max_bytes,
      #[cfg(feature = "regex")]
      pattern: self.pattern,
      prefix: self.prefix,
      suffix: self.suffix,
      contains: self.contains,
      not_contains: Some(val.into()),
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn in_(self, val: &'static SortedList<&'static str>) -> StringValidatorBuilder<SetIn<S>>
  where
    S::In: IsUnset,
  {
    StringValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      well_known: self.well_known,
      ignore: self.ignore,
      required: self.required,
      len: self.len,
      min_len: self.min_len,
      max_len: self.max_len,
      len_bytes: self.len_bytes,
      min_bytes: self.min_bytes,
      max_bytes: self.max_bytes,
      #[cfg(feature = "regex")]
      pattern: self.pattern,
      prefix: self.prefix,
      suffix: self.suffix,
      contains: self.contains,
      not_contains: self.not_contains,
      in_: Some(val),
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn not_in(self, val: &'static SortedList<&'static str>) -> StringValidatorBuilder<SetNotIn<S>>
  where
    S::NotIn: IsUnset,
  {
    StringValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      well_known: self.well_known,
      ignore: self.ignore,
      required: self.required,
      len: self.len,
      min_len: self.min_len,
      max_len: self.max_len,
      len_bytes: self.len_bytes,
      min_bytes: self.min_bytes,
      max_bytes: self.max_bytes,
      #[cfg(feature = "regex")]
      pattern: self.pattern,
      prefix: self.prefix,
      suffix: self.suffix,
      contains: self.contains,
      not_contains: self.not_contains,
      in_: self.in_,
      not_in: Some(val),
      const_: self.const_,
    }
  }

  pub fn const_<T: Into<Arc<str>>>(self, val: T) -> StringValidatorBuilder<SetConst<S>>
  where
    S::Const: IsUnset,
  {
    StringValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      well_known: self.well_known,
      ignore: self.ignore,
      required: self.required,
      len: self.len,
      min_len: self.min_len,
      max_len: self.max_len,
      len_bytes: self.len_bytes,
      min_bytes: self.min_bytes,
      max_bytes: self.max_bytes,
      #[cfg(feature = "regex")]
      pattern: self.pattern,
      prefix: self.prefix,
      suffix: self.suffix,
      contains: self.contains,
      not_contains: self.not_contains,
      in_: self.in_,
      not_in: self.not_in,
      const_: Some(val.into()),
    }
  }

  pub fn build(self) -> StringValidator {
    StringValidator {
      cel: self.cel,
      well_known: self.well_known,
      ignore: self.ignore,
      required: self.required,
      len: self.len,
      min_len: self.min_len,
      max_len: self.max_len,
      len_bytes: self.len_bytes,
      min_bytes: self.min_bytes,
      max_bytes: self.max_bytes,
      #[cfg(feature = "regex")]
      pattern: self.pattern,
      prefix: self.prefix,
      suffix: self.suffix,
      contains: self.contains,
      not_contains: self.not_contains,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }
}
