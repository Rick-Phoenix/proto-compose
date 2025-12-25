#[macro_use]
mod macros;

pub use prost;
use std::fmt::Display;

use askama::Template;
use fxhash::FxHashMap;
#[doc(hidden)]
pub use inventory;
#[cfg(feature = "testing")]
use owo_colors::OwoColorize;
#[doc(hidden)]
pub use paste::paste;
use proto_types::protovalidate::{FieldPathElement, Violations};
pub use protocheck_core::validators::containing::SortedList;
use thiserror::Error;
mod oneof;
mod options;
mod validators;
use std::{collections::HashSet, fmt::Write, marker::PhantomData, ops::Range, sync::LazyLock};
mod field;
mod file;
mod message;
mod optional;
mod package;
mod proto_enum;
mod proto_type;
mod rendering_utils;
mod service;
#[cfg(feature = "testing")]
pub mod test_utils;
mod well_known_types;
use bon::Builder;
pub use field::*;
pub use file::*;
pub use message::*;
pub use oneof::*;
pub use options::*;
pub use package::*;
pub use proto_enum::*;
pub use proto_type::*;
pub use protocheck_core::field_data::FieldContext;
use protocheck_core::field_data::FieldKind;
use rendering_utils::*;
pub use service::*;
pub use validators::*;
mod registry;
pub use registry::*;

#[doc(hidden)]
pub fn apply<I, O, F>(input: I, f: F) -> O
where
  F: FnOnce(I) -> O,
{
  f(input)
}
