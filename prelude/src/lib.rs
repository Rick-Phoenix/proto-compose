#[macro_use]
mod macros;

use askama::Template;
pub use paste::paste;
mod oneof;
mod options;
mod validators;
use std::{collections::HashSet, ops::Range};
mod field;
mod file;
mod message;
mod optional;
mod proto_enum;
mod proto_type;
mod rendering_utils;
mod well_known_types;

use bon::Builder;
pub use field::*;
pub use file::*;
pub use message::*;
pub use oneof::*;
pub use options::*;
pub use proto_enum::*;
pub use proto_type::*;
use rendering_utils::*;
pub use validators::*;

#[doc(hidden)]
pub fn apply<I, O, F>(input: I, f: F) -> O
where
  F: FnOnce(I) -> O,
{
  f(input)
}
