#[macro_use]
mod macros;

use askama::Template;
#[doc(hidden)]
pub use paste::paste;
mod oneof;
mod options;
mod validators;
use std::{
  collections::{HashMap, HashSet},
  fmt::Write,
  marker::PhantomData,
  ops::Range,
  sync::{Arc, LazyLock},
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
