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
  ops::DerefMut,
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

#[cfg(not(feature = "std"))]
mod lazy;
#[cfg(not(feature = "std"))]
pub use lazy::Lazy;

#[cfg(feature = "std")]
pub use std::sync::LazyLock as Lazy;

#[doc(hidden)]
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

  macro_rules! impl_into_regex {
    ($(( $trait:ident, $path:ident )),*) => {
      $(
        pub trait $trait {
          #[allow(private_interfaces)]
          const SEALED: Sealed;

          fn into_regex(self) -> Cow<'static, $path>;
        }

        impl $trait for &str {
          #[allow(private_interfaces)]
          const SEALED: Sealed = Sealed;

          #[inline]
          fn into_regex(self) -> Cow<'static, $path> {
            Cow::Owned($path::new(self).unwrap())
          }
        }

        impl $trait for Arc<str> {
          #[allow(private_interfaces)]
          const SEALED: Sealed = Sealed;

          #[inline]
          fn into_regex(self) -> Cow<'static, $path> {
            Cow::Owned($path::new(&self).unwrap())
          }
        }

        impl $trait for $path {
          #[allow(private_interfaces)]
          const SEALED: Sealed = Sealed;

          #[inline]
          fn into_regex(self) -> Cow<'static, $path> {
            Cow::Owned(self)
          }
        }

        impl $trait for &'static $path {
          #[allow(private_interfaces)]
          const SEALED: Sealed = Sealed;

          #[inline]
          fn into_regex(self) -> Cow<'static, $path> {
            Cow::Borrowed(self)
          }
        }
      )*
    };
  }

  impl_into_regex!((IntoRegex, Regex), (IntoBytesRegex, BytesRegex));
}
