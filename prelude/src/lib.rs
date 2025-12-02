#[macro_use]
mod macros;

pub use paste::paste;
mod items;
mod oneof;
pub mod validators;
use std::{
  collections::{BTreeSet, HashSet},
  ops::Range,
  sync::Arc,
};
mod field;
mod file;
mod message;
mod optional;
mod proto_enum;
mod proto_type;

use bon::Builder;
pub use field::*;
pub use file::*;
pub use items::*;
pub use message::*;
pub use oneof::*;
pub use proto_enum::*;
pub use proto_type::*;

#[doc(hidden)]
pub fn apply<I, O, F>(input: I, f: F) -> O
where
  F: FnOnce(I) -> O,
{
  f(input)
}
