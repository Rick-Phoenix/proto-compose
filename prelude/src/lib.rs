#[macro_use]
mod macros;

use askama::Template;
#[doc(hidden)]
pub use paste::paste;
use proto_types::protovalidate::{FieldPathElement, Violations};
mod oneof;
mod options;
mod validators;
use std::{
  collections::{HashMap, HashSet},
  fmt::Write,
  marker::PhantomData,
  ops::Range,
  sync::LazyLock,
};
mod field;
mod file;
mod message;
mod optional;
mod package;
mod proto_enum;
mod proto_type;
mod rendering_utils;
mod service;
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
use protocheck_core::field_data::FieldKind;
pub use protocheck_core::{field_data::FieldContext, validators::containing::ItemLookup};
use rendering_utils::*;
pub use service::*;
pub use validators::*;

#[doc(hidden)]
pub fn apply<I, O, F>(input: I, f: F) -> O
where
  F: FnOnce(I) -> O,
{
  f(input)
}
