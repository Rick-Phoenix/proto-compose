pub mod state;
use crate::validators::*;
pub use state::*;

use ::bytes::Bytes;
#[cfg(feature = "regex")]
use regex::bytes::Regex;

#[derive(Clone, Debug, Default)]
pub struct BytesValidatorBuilder<S: State = Empty> {
  _state: PhantomData<S>,

  /// Adds custom validation using one or more [`CelRule`]s to this field.
  cel: Vec<&'static CelProgram>,

  ignore: Option<Ignore>,

  well_known: Option<WellKnownBytes>,

  /// Specifies that the field must be set in order to be valid.
  required: bool,

  /// Specifies that the given `bytes` field must be of this exact length.
  len: Option<usize>,

  /// Specifies that the given `bytes` field must have a length that is equal to or higher than the given value.
  min_len: Option<usize>,

  /// Specifies that the given `bytes` field must have a length that is equal to or lower than the given value.
  max_len: Option<usize>,

  #[cfg(feature = "regex")]
  /// Specifies a regex pattern that must be matches by the value to pass validation.
  pattern: Option<&'static Regex>,

  /// Specifies a prefix that the value must start with in order to pass validation.
  prefix: Option<Bytes>,

  /// Specifies a suffix that the value must end with in order to pass validation.
  suffix: Option<Bytes>,

  /// Specifies a subset of bytes that the value must contain in order to pass validation.
  contains: Option<Bytes>,

  /// Specifies that only the values in this list will be considered valid for this field.
  in_: Option<&'static SortedList<&'static [u8]>>,

  /// Specifies that the values in this list will be considered NOT valid for this field.
  not_in: Option<&'static SortedList<&'static [u8]>>,

  /// Specifies that only this specific value will be considered valid for this field.
  const_: Option<Bytes>,
}

impl<S: State> From<BytesValidatorBuilder<S>> for ProtoOption {
  fn from(value: BytesValidatorBuilder<S>) -> Self {
    value.build().into()
  }
}

impl BytesValidator {
  #[must_use]
  pub fn builder() -> BytesValidatorBuilder {
    BytesValidatorBuilder::default()
  }
}

macro_rules! well_known_impl {
  ($name:ident, $doc:literal) => {
    paste::paste! {
      #[doc = $doc]
      pub fn [< $name:snake >](self) -> BytesValidatorBuilder<SetWellKnown<S>>
        where
          S::WellKnown: IsUnset,
        {
          BytesValidatorBuilder {
            _state: PhantomData,
            cel: self.cel,
            ignore: self.ignore,
            well_known: Some(WellKnownBytes::$name),
            required: self.required,
            len: self.len,
            min_len: self.min_len,
            max_len: self.max_len,
            #[cfg(feature = "regex")]
            pattern: self.pattern,
            prefix: self.prefix,
            suffix: self.suffix,
            contains: self.contains,
            in_: self.in_,
            not_in: self.not_in,
            const_: self.const_,
          }
        }
    }
  };
}

impl<S: State> BytesValidatorBuilder<S> {
  well_known_impl!(
    Ip,
    "Specifies that the value must be a valid IP address (v4 or v6) in byte format."
  );
  well_known_impl!(
    Ipv4,
    "Specifies that the value must be a valid IPv4 address in byte format."
  );
  well_known_impl!(
    Ipv6,
    "Specifies that the value must be a valid IPv6 address in byte format."
  );
}

#[allow(
  clippy::must_use_candidate,
  clippy::use_self,
  clippy::return_self_not_must_use
)]
impl<S: State> BytesValidatorBuilder<S> {
  pub fn ignore_always(self) -> BytesValidatorBuilder<SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    BytesValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: Some(Ignore::Always),
      well_known: self.well_known,
      required: self.required,
      len: self.len,
      min_len: self.min_len,
      max_len: self.max_len,
      #[cfg(feature = "regex")]
      pattern: self.pattern,
      prefix: self.prefix,
      suffix: self.suffix,
      contains: self.contains,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn ignore_if_zero_value(self) -> BytesValidatorBuilder<SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    BytesValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: Some(Ignore::IfZeroValue),
      well_known: self.well_known,
      required: self.required,
      len: self.len,
      min_len: self.min_len,
      max_len: self.max_len,
      #[cfg(feature = "regex")]
      pattern: self.pattern,
      prefix: self.prefix,
      suffix: self.suffix,
      contains: self.contains,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn cel(mut self, program: &'static CelProgram) -> BytesValidatorBuilder<S> {
    self.cel.push(program);

    BytesValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      well_known: self.well_known,
      required: self.required,
      len: self.len,
      min_len: self.min_len,
      max_len: self.max_len,
      #[cfg(feature = "regex")]
      pattern: self.pattern,
      prefix: self.prefix,
      suffix: self.suffix,
      contains: self.contains,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn len(self, val: usize) -> BytesValidatorBuilder<SetLen<S>>
  where
    S::Len: IsUnset,
  {
    BytesValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      well_known: self.well_known,
      required: self.required,
      len: Some(val),
      min_len: self.min_len,
      max_len: self.max_len,
      #[cfg(feature = "regex")]
      pattern: self.pattern,
      prefix: self.prefix,
      suffix: self.suffix,
      contains: self.contains,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn min_len(self, val: usize) -> BytesValidatorBuilder<SetMinLen<S>>
  where
    S::MinLen: IsUnset,
  {
    BytesValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      well_known: self.well_known,
      required: self.required,
      len: self.len,
      min_len: Some(val),
      max_len: self.max_len,
      #[cfg(feature = "regex")]
      pattern: self.pattern,
      prefix: self.prefix,
      suffix: self.suffix,
      contains: self.contains,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn max_len(self, val: usize) -> BytesValidatorBuilder<SetMaxLen<S>>
  where
    S::MaxLen: IsUnset,
  {
    BytesValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      well_known: self.well_known,
      required: self.required,
      len: self.len,
      min_len: self.min_len,
      max_len: Some(val),
      #[cfg(feature = "regex")]
      pattern: self.pattern,
      prefix: self.prefix,
      suffix: self.suffix,
      contains: self.contains,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  #[cfg(feature = "regex")]
  pub fn pattern(self, val: &'static Regex) -> BytesValidatorBuilder<SetPattern<S>>
  where
    S::Pattern: IsUnset,
  {
    BytesValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      well_known: self.well_known,
      required: self.required,
      len: self.len,
      min_len: self.min_len,
      max_len: self.max_len,
      pattern: Some(val),
      prefix: self.prefix,
      suffix: self.suffix,
      contains: self.contains,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn prefix<T: Into<Bytes>>(self, val: T) -> BytesValidatorBuilder<SetPrefix<S>>
  where
    S::Prefix: IsUnset,
  {
    BytesValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      well_known: self.well_known,
      required: self.required,
      len: self.len,
      min_len: self.min_len,
      max_len: self.max_len,
      #[cfg(feature = "regex")]
      pattern: self.pattern,
      prefix: Some(val.into()),
      suffix: self.suffix,
      contains: self.contains,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn suffix<T: Into<Bytes>>(self, val: T) -> BytesValidatorBuilder<SetSuffix<S>>
  where
    S::Suffix: IsUnset,
  {
    BytesValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      well_known: self.well_known,
      required: self.required,
      len: self.len,
      min_len: self.min_len,
      max_len: self.max_len,
      #[cfg(feature = "regex")]
      pattern: self.pattern,
      prefix: self.prefix,
      suffix: Some(val.into()),
      contains: self.contains,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn contains<T: Into<Bytes>>(self, val: T) -> BytesValidatorBuilder<SetContains<S>>
  where
    S::Contains: IsUnset,
  {
    BytesValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      well_known: self.well_known,
      required: self.required,
      len: self.len,
      min_len: self.min_len,
      max_len: self.max_len,
      #[cfg(feature = "regex")]
      pattern: self.pattern,
      prefix: self.prefix,
      suffix: self.suffix,
      contains: Some(val.into()),
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn not_in(
    self,
    list: &'static SortedList<&'static [u8]>,
  ) -> BytesValidatorBuilder<SetNotIn<S>>
  where
    S::NotIn: IsUnset,
  {
    BytesValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      well_known: self.well_known,
      required: self.required,
      len: self.len,
      min_len: self.min_len,
      max_len: self.max_len,
      #[cfg(feature = "regex")]
      pattern: self.pattern,
      prefix: self.prefix,
      suffix: self.suffix,
      contains: self.contains,
      in_: self.in_,
      not_in: Some(list),
      const_: self.const_,
    }
  }

  pub fn in_(self, list: &'static SortedList<&'static [u8]>) -> BytesValidatorBuilder<SetIn<S>>
  where
    S::In: IsUnset,
  {
    BytesValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      well_known: self.well_known,
      required: self.required,
      len: self.len,
      min_len: self.min_len,
      max_len: self.max_len,
      #[cfg(feature = "regex")]
      pattern: self.pattern,
      prefix: self.prefix,
      suffix: self.suffix,
      contains: self.contains,
      in_: Some(list),
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn const_<T: Into<Bytes>>(self, val: T) -> BytesValidatorBuilder<SetConst<S>>
  where
    S::Const: IsUnset,
  {
    BytesValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      well_known: self.well_known,
      required: self.required,
      len: self.len,
      min_len: self.min_len,
      max_len: self.max_len,
      #[cfg(feature = "regex")]
      pattern: self.pattern,
      prefix: self.prefix,
      suffix: self.suffix,
      contains: self.contains,
      in_: self.in_,
      not_in: self.not_in,
      const_: Some(val.into()),
    }
  }

  pub fn required(self) -> BytesValidatorBuilder<SetRequired<S>>
  where
    S::Required: IsUnset,
  {
    BytesValidatorBuilder {
      _state: PhantomData,
      cel: self.cel,
      ignore: self.ignore,
      well_known: self.well_known,
      required: true,
      len: self.len,
      min_len: self.min_len,
      max_len: self.max_len,
      #[cfg(feature = "regex")]
      pattern: self.pattern,
      prefix: self.prefix,
      suffix: self.suffix,
      contains: self.contains,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }

  pub fn build(self) -> BytesValidator {
    BytesValidator {
      cel: self.cel,
      ignore: self.ignore,
      well_known: self.well_known,
      required: self.required,
      len: self.len,
      min_len: self.min_len,
      max_len: self.max_len,
      #[cfg(feature = "regex")]
      pattern: self.pattern,
      prefix: self.prefix,
      suffix: self.suffix,
      contains: self.contains,
      in_: self.in_,
      not_in: self.not_in,
      const_: self.const_,
    }
  }
}
