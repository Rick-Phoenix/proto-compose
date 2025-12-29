#![allow(clippy::must_use_candidate)]

#[cfg(test)]
mod tests;

use std::collections::HashMap;

use prelude::*;
use proc_macro_impls::Oneof;
use proto_types::{Duration, Timestamp};

pub fn collect_package() -> Package {
  prelude::collect_package("myapp.v1")
}

pub mod inner {
  use bytes::Bytes;
  use proc_macro_impls::{
    Extension, proto_enum, proto_extension, proto_message, proto_oneof, proto_service,
  };

  use super::*;

  proto_file!("abc.proto", package = "myapp.v1");

  #[proto_extension(target = MessageOptions)]
  pub struct SomeExt {
    #[proto(tag = 5000)]
    name: String,

    #[proto(tag = 5001)]
    name2: String,
  }

  #[proto_service]
  pub enum FrodoService {
    GetRing { request: Abc, response: Nested },
    DestroyRing { request: Abc, response: Nested },
  }

  #[proto_enum]
  #[proto(reserved_numbers(1, 2, 10..MAX))]
  #[proto(reserved_names("abc", "bcd"))]
  pub enum PseudoEnum {
    AbcDeg,
    B,
    C,
  }

  #[proto_oneof]
  #[proto(required)]
  #[derive(Clone, Debug)]
  pub enum PseudoOneof {
    #[proto(tag = 200, validate = |v| v.min_len(10).max_len(50))]
    A(String),
    #[proto(tag = 201, validate = |v| v.gt(0).lt(50))]
    B(i32),
    #[proto(tag = 202, message(proxied, boxed))]
    C(Box<Abc>),
  }

  #[proto_oneof(direct)]
  #[proto(required)]
  pub enum DirectOneof {
    #[proto(tag = 200)]
    A(String),
    #[proto(tag = 201)]
    B(i32),
    #[proto(tag = 202, message(proxied, boxed))]
    C(Box<Abc>),
  }

  impl Default for PseudoOneofProto {
    fn default() -> Self {
      Self::B(0)
    }
  }

  impl Default for PseudoOneof {
    fn default() -> Self {
      Self::B(0)
    }
  }

  #[proto_message(no_auto_test)]
  #[proto(reserved_numbers(1, 2, 3..9))]
  #[proto(reserved_names("abc", "bcd"))]
  #[derive(Clone, Debug, Default)]
  pub struct Abc {
    #[proto(repeated(float), validate = |v| v.unique())]
    pub repeated_float: Vec<f32>,

    #[proto(timestamp, validate = |v| v.lt_now())]
    pub timestamp: Option<Timestamp>,

    #[proto(duration, validate = |v| v.lt(Duration { seconds: 2000, nanos: 0 }))]
    duration: Option<Duration>,

    #[proto(message(AbcProto, boxed))]
    boxed: Option<Box<Self>>,

    #[proto(bytes, validate = |v| v.pattern(inline_bytes_regex!("abc", "abc")))]
    pub bytes: Bytes,

    #[proto(tag = 35, validate = |v| v.pattern(inline_regex!("abc", "abc")).in_(inline_static_list!(&str, ["abc"])))]
    name: String,

    #[proto(validate = |v| v.min_pairs(0).keys(|k| k.min_len(25)).values(|v| v.lt(25)))]
    map: HashMap<String, i32>,

    #[proto(map(string, enum_), validate = |v| v.min_pairs(20).values(|val| val.defined_only()))]
    enum_map: HashMap<String, PseudoEnum>,

    #[proto(map(string, message(proxied)))]
    message_map: HashMap<String, Nested>,

    #[proto(enum_, validate = |v| v.defined_only())]
    enum_field: PseudoEnum,

    #[proto(repeated(enum_), validate = |v| v.unique())]
    pub repeated_enum: Vec<PseudoEnum>,

    #[proto(enum_)]
    optional_enum: Option<PseudoEnum>,

    #[proto(message(proxied))]
    nested: Option<Nested>,

    #[proto(repeated(message(proxied)))]
    repeated_message: Vec<Nested>,

    #[proto(oneof(default, proxied, tags(200, 201, 202)))]
    oneof: PseudoOneof,

    #[proto(sint32)]
    sint32: i32,

    #[proto(repeated(sint32), validate = |v| v.items(|it| it.gt(0)))]
    pub sint32_repeated: Vec<i32>,

    #[proto(map(sint32, uint32), validate = |v| v.keys(|k| k.gt(0)).values(|vals| vals.gt(0)))]
    sint32_map: HashMap<i32, u32>,

    #[proto(sint32)]
    sint32_optional: Option<i32>,
  }

  #[proto_message]
  #[proto(parent_message = Abc)]
  #[derive(Clone, Debug)]
  pub struct Nested {
    name: String,
  }

  #[proto_message(direct)]
  #[proto(parent_message = Nested)]
  pub struct Nested2 {
    name: String,

    num: i32,

    #[proto(oneof(proxied, tags(200, 201, 202)))]
    reused_oneof: Option<PseudoOneof>,
  }
}
