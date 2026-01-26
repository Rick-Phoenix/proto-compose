// no_std support is planned at a certain point, but not yet implemented
#![no_std]
#![deny(clippy::alloc_instead_of_core)]
#![deny(clippy::std_instead_of_alloc)]
#![deny(clippy::std_instead_of_core)]

#[cfg(any(test, feature = "std"))]
extern crate std;

pub use alloc::{
  boxed::Box, collections::BTreeMap, format, string::String, string::ToString, sync::Arc, vec,
  vec::Vec,
};

#[doc(hidden)]
pub extern crate alloc;

#[cfg(feature = "cel")]
pub use ::cel;

#[macro_use]
mod decl_macros;

use ::bytes::Bytes;
use alloc::{borrow::Cow, borrow::ToOwned, collections::BTreeSet};
use core::borrow::Borrow;
use core::fmt::{Debug, Display};
use core::{
  fmt::Write,
  hash::Hash,
  marker::{PhantomData, Sized},
  ops::Deref,
  ops::Range,
};

#[cfg(not(feature = "std"))]
use hashbrown::HashMap;

#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(feature = "std")]
use askama::Template;
use float_eq::{FloatEq, float_eq};
#[doc(hidden)]
#[cfg(feature = "inventory")]
pub use inventory;
use ordered_float::{FloatCore, OrderedFloat};
use owo_colors::OwoColorize;
#[doc(hidden)]
use paste::paste;
pub use proc_macro_impls as macros;
pub use proc_macro_impls::*;
pub use proto_types;
pub use proto_types::protovalidate::{FieldPathElement, Violations};
use thiserror::Error;
mod oneof;
mod options;
pub mod validators;
#[cfg(not(feature = "std"))]
use hashbrown::HashSet;
#[cfg(feature = "std")]
use std::collections::HashSet;
mod field;
mod file;
mod package;
mod proto_enum;
mod proto_message;
mod proto_type;
mod rendering_utils;
mod service;
pub mod test_utils;
pub use test_utils::*;
mod well_known_types;
pub use field::*;
pub use file::*;
pub use oneof::*;
pub use options::*;
pub use package::*;
pub use proto_enum::*;
pub use proto_message::*;
pub use proto_type::*;
use rendering_utils::*;
pub use service::*;
pub use validators::*;
mod registry;
pub use registry::*;
mod extension;
pub use extension::*;

#[cfg(feature = "serde")]
pub(crate) mod serde_impls;

#[cfg(not(feature = "std"))]
mod lazy;
#[cfg(not(feature = "std"))]
pub use lazy::Lazy;

#[cfg(feature = "std")]
pub use std::sync::LazyLock as Lazy;

#[doc(hidden)]
#[inline]
pub fn apply<I, O, F>(input: I, f: F) -> O
where
  F: FnOnce(I) -> O,
{
  f(input)
}

#[doc(hidden)]
#[allow(clippy::wrong_self_convention)]
pub trait IntoBytes {
  #[allow(private_interfaces)]
  const SEALED: Sealed;

  fn into_bytes(self) -> Bytes;
}

impl<const N: usize> IntoBytes for &'static [u8; N] {
  #[allow(private_interfaces)]
  const SEALED: Sealed = Sealed;

  #[inline]
  fn into_bytes(self) -> Bytes {
    Bytes::from_static(self)
  }
}

impl IntoBytes for &'static [u8] {
  #[allow(private_interfaces)]
  const SEALED: Sealed = Sealed;

  #[inline]
  fn into_bytes(self) -> Bytes {
    Bytes::from_static(self)
  }
}

impl IntoBytes for Bytes {
  #[allow(private_interfaces)]
  const SEALED: Sealed = Sealed;

  #[inline]
  fn into_bytes(self) -> Bytes {
    self
  }
}

impl IntoBytes for &Bytes {
  #[allow(private_interfaces)]
  const SEALED: Sealed = Sealed;

  #[inline]
  fn into_bytes(self) -> Bytes {
    self.clone()
  }
}

#[cfg(feature = "regex")]
pub use regex_impls::*;

#[cfg(feature = "regex")]
mod regex_impls {
  use super::*;

  use regex::Regex;
  use regex::bytes::Regex as BytesRegex;

  pub trait IntoRegex {
    #[allow(private_interfaces)]
    const SEALED: Sealed;

    fn into_regex(self) -> Regex;
  }

  impl IntoRegex for &str {
    #[allow(private_interfaces)]
    const SEALED: Sealed = Sealed;

    #[track_caller]
    #[inline]
    fn into_regex(self) -> Regex {
      Regex::new(self).unwrap()
    }
  }

  impl IntoRegex for Regex {
    #[allow(private_interfaces)]
    const SEALED: Sealed = Sealed;

    fn into_regex(self) -> Regex {
      self
    }
  }

  impl IntoRegex for &Regex {
    #[allow(private_interfaces)]
    const SEALED: Sealed = Sealed;

    #[inline]
    fn into_regex(self) -> Regex {
      self.clone()
    }
  }

  pub trait IntoBytesRegex {
    #[allow(private_interfaces)]
    const SEALED: Sealed;

    fn into_regex(self) -> BytesRegex;
  }

  impl IntoBytesRegex for &str {
    #[allow(private_interfaces)]
    const SEALED: Sealed = Sealed;

    #[track_caller]
    #[inline]
    fn into_regex(self) -> BytesRegex {
      BytesRegex::new(self).unwrap()
    }
  }

  impl IntoBytesRegex for BytesRegex {
    #[allow(private_interfaces)]
    const SEALED: Sealed = Sealed;

    #[inline]
    fn into_regex(self) -> BytesRegex {
      self
    }
  }

  impl IntoBytesRegex for &BytesRegex {
    #[allow(private_interfaces)]
    const SEALED: Sealed = Sealed;

    #[inline]
    fn into_regex(self) -> BytesRegex {
      self.clone()
    }
  }
}

#[derive(Clone, Default, Debug, Copy)]
pub struct OneofValidator {
  pub required: bool,
}

impl<T: ValidatedOneof + ProtoValidation> ValidatorBuilderFor<T> for OneofValidator {
  type Target = T;
  type Validator = Self;

  #[inline]
  fn build_validator(self) -> Self::Validator {
    self
  }
}

impl<T: ValidatedOneof + ProtoValidation> Validator<T> for OneofValidator {
  type Target = T;

  // Should be inlined because if the assoc. constant is false, it may promote
  // dead code elimination
  #[inline]
  fn validate_core<V>(&self, ctx: &mut ValidationCtx, val: Option<&V>) -> ValidatorResult
  where
    V: Borrow<Self::Target> + ?Sized,
  {
    match val {
      Some(oneof) => {
        if T::HAS_DEFAULT_VALIDATOR {
          oneof.borrow().validate(ctx)
        } else {
          Ok(IsValid::Yes)
        }
      }
      None => {
        if self.required {
          ctx.add_required_oneof_violation()
        } else {
          Ok(IsValid::Yes)
        }
      }
    }
  }
}

impl OneofValidator {
  #[must_use]
  #[inline]
  pub const fn new(required: bool) -> Self {
    Self { required }
  }
}

#[cfg(feature = "serde")]
pub trait MaybeSerde: serde::Serialize + serde::de::DeserializeOwned {}

#[cfg(feature = "serde")]
impl<T: serde::Serialize + serde::de::DeserializeOwned> MaybeSerde for T {}

#[cfg(not(feature = "serde"))]
pub trait MaybeSerde {}

#[cfg(not(feature = "serde"))]
impl<T> MaybeSerde for T {}
