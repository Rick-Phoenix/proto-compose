#[doc(hidden)]
pub mod state;
use crate::validators::*;
pub(crate) use state::*;

use ::bytes::Bytes;

#[derive(Clone, Debug)]
pub struct BytesValidatorBuilder<S: State = Empty> {
  _state: PhantomData<S>,
  data: BytesValidator,
}

impl ProtoValidator for Bytes {
  type Target = Self;
  type Stored = Self;
  type Validator = BytesValidator;
  type Builder = BytesValidatorBuilder;

  type UniqueStore<'a>
    = RefHybridStore<'a, Self>
  where
    Self: 'a;

  #[inline]
  fn make_unique_store<'a>(_: &Self::Validator, cap: usize) -> Self::UniqueStore<'a> {
    RefHybridStore::default_with_capacity(cap)
  }
}
impl<S: State> ValidatorBuilderFor<Bytes> for BytesValidatorBuilder<S> {
  type Target = Bytes;
  type Validator = BytesValidator;
  #[inline]
  fn build_validator(self) -> BytesValidator {
    self.build()
  }
}

impl<S: State> Default for BytesValidatorBuilder<S> {
  #[inline]
  fn default() -> Self {
    Self {
      _state: PhantomData,
      data: BytesValidator::default(),
    }
  }
}

impl<S: State> From<BytesValidatorBuilder<S>> for ProtoOption {
  fn from(value: BytesValidatorBuilder<S>) -> Self {
    value.build().into()
  }
}

impl BytesValidator {
  #[must_use]
  #[inline]
  pub fn builder() -> BytesValidatorBuilder {
    BytesValidatorBuilder::default()
  }
}

macro_rules! well_known_impl {
  ($name:ident, $doc:literal) => {
    paste::paste! {
      #[inline]
      #[doc = $doc]
      pub fn [< $name:snake >](mut self) -> BytesValidatorBuilder<SetWellKnown<S>>
        where
          S::WellKnown: IsUnset,
        {
          self.data.well_known = Some(WellKnownBytes::$name);

          BytesValidatorBuilder {
            _state: PhantomData,
            data: self.data
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
  #[cfg(feature = "regex")]
  well_known_impl!(
    Uuid,
    "Specifies that the value must be a valid UUID in byte format."
  );
}

#[allow(
  clippy::must_use_candidate,
  clippy::use_self,
  clippy::return_self_not_must_use
)]
impl<S: State> BytesValidatorBuilder<S> {
  custom_error_messages_method!(Bytes);

  #[inline]
  pub fn ignore_always(mut self) -> BytesValidatorBuilder<SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    self.data.ignore = Ignore::Always;

    BytesValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn ignore_if_zero_value(mut self) -> BytesValidatorBuilder<SetIgnore<S>>
  where
    S::Ignore: IsUnset,
  {
    self.data.ignore = Ignore::IfZeroValue;

    BytesValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn cel(mut self, program: CelProgram) -> BytesValidatorBuilder<S> {
    self.data.cel.push(program);

    BytesValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn len(mut self, val: usize) -> BytesValidatorBuilder<SetLen<S>>
  where
    S::Len: IsUnset,
  {
    self.data.len = Some(val);

    BytesValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn min_len(mut self, val: usize) -> BytesValidatorBuilder<SetMinLen<S>>
  where
    S::MinLen: IsUnset,
  {
    self.data.min_len = Some(val);

    BytesValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn max_len(mut self, val: usize) -> BytesValidatorBuilder<SetMaxLen<S>>
  where
    S::MaxLen: IsUnset,
  {
    self.data.max_len = Some(val);

    BytesValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[cfg(feature = "regex")]
  #[inline]
  pub fn pattern(mut self, val: impl IntoBytesRegex) -> BytesValidatorBuilder<SetPattern<S>>
  where
    S::Pattern: IsUnset,
  {
    self.data.pattern = Some(val.into_regex());

    BytesValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn prefix(mut self, val: impl IntoBytes) -> BytesValidatorBuilder<SetPrefix<S>>
  where
    S::Prefix: IsUnset,
  {
    self.data.prefix = Some(val.into_bytes());

    BytesValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn suffix(mut self, val: impl IntoBytes) -> BytesValidatorBuilder<SetSuffix<S>>
  where
    S::Suffix: IsUnset,
  {
    self.data.suffix = Some(val.into_bytes());

    BytesValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn contains(mut self, val: impl IntoBytes) -> BytesValidatorBuilder<SetContains<S>>
  where
    S::Contains: IsUnset,
  {
    self.data.contains = Some(val.into_bytes());

    BytesValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn not_in(mut self, list: impl IntoSortedList<Bytes>) -> BytesValidatorBuilder<SetNotIn<S>>
  where
    S::NotIn: IsUnset,
  {
    self.data.not_in = Some(list.into_sorted_list());

    BytesValidatorBuilder {
      _state: PhantomData,

      data: self.data,
    }
  }

  #[inline]
  pub fn in_(mut self, list: impl IntoSortedList<Bytes>) -> BytesValidatorBuilder<SetIn<S>>
  where
    S::In: IsUnset,
  {
    self.data.in_ = Some(list.into_sorted_list());

    BytesValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn const_(mut self, val: impl IntoBytes) -> BytesValidatorBuilder<SetConst<S>>
  where
    S::Const: IsUnset,
  {
    self.data.const_ = Some(val.into_bytes());

    BytesValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn required(mut self) -> BytesValidatorBuilder<SetRequired<S>>
  where
    S::Required: IsUnset,
  {
    self.data.required = true;

    BytesValidatorBuilder {
      _state: PhantomData,
      data: self.data,
    }
  }

  #[inline]
  pub fn build(self) -> BytesValidator {
    self.data
  }
}
