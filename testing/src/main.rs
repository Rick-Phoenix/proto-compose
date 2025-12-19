#![allow(unused)]

use std::collections::HashMap;

use prelude::{
  cel_program, CachedProgram, EnumValidator, FieldValidator, IntValidator, MessageEntry,
  OptionValue, Package, ProtoEnum, ProtoOption, RepeatedValidator, RepeatedValidatorBuilder,
  StringValidator, StringValidatorBuilder, ValidatorBuilderFor,
};
use proc_macro_impls::{Enum, Message, Oneof};
use proto_types::{Duration, Timestamp};

fn string_validator() -> StringValidatorBuilder {
  StringValidator::builder()
}

fn repeated_validator() -> impl ValidatorBuilderFor<Vec<i32>> {
  let validator: RepeatedValidatorBuilder<i32> = RepeatedValidator::builder();

  validator.items(|i| i.lt(20)).min_items(1)
}

fn enum_validator<T: ProtoEnum>() -> impl ValidatorBuilderFor<T, Target = i32> {
  let validator = EnumValidator::builder();

  validator.defined_only()
}

fn numeric_validator() -> impl ValidatorBuilderFor<Sint32, Target = i32> {
  let validator = IntValidator::builder();

  validator.lt(0)
}

fn random_option() -> ProtoOption {
  ProtoOption {
    name: "(hobbits.location)".into(),
    value: OptionValue::String("isengard".into()),
  }
}

#[proc_macro_impls::proto_module(file = "abc.proto", package = "myapp.v1")]
mod inner {
  use std::{
    collections::HashSet,
    sync::{Arc, LazyLock},
  };

  use bytes::Bytes;
  use prelude::{bytes_regex, cached_set, CachedBytesRegex, *};
  use proc_macro_impls::{
    proto_enum, proto_extension, proto_message, proto_oneof, proto_service, Extension, Service,
  };
  use proto_types::{field_descriptor_proto::Type, protovalidate::FieldPathElement};
  use regex::Regex;

  use super::*;

  static RULE: CachedProgram = cel_program!(id = "abc", msg = "abc", expr = "abc");

  #[proto_extension(target = MessageOptions)]
  struct SomeExt {
    #[proto(tag = 5000)]
    name: String,
  }

  #[proto_service]
  #[proto(options = vec![ random_option() ])]
  enum FrodoService {
    #[proto(options = vec![ random_option() ])]
    GetRing {
      request: Abc,
      response: Nested,
    },
    DestroyRing {
      request: Abc,
      response: Nested,
    },
  }

  #[proto_enum]
  #[proto(reserved_numbers(1, 2, 10..MAX))]
  #[proto(reserved_names("abc", "bcd"))]
  #[proto(options = vec![ random_option() ])]
  #[derive(Clone, Debug)]
  pub enum PseudoEnum {
    AbcDeg,
    B,
    C,
  }

  #[proto_oneof]
  #[proto(required)]
  #[derive(Clone, Debug)]
  enum PseudoOneof {
    #[proto(tag = 200, validate = |v| v.cel([ &RULE ]))]
    A(String),
    #[proto(tag = 201)]
    B(i32),
    #[proto(tag = 202, message(proxied, boxed))]
    C(Box<Abc>),
  }

  #[proto_oneof(direct)]
  #[proto(required)]
  enum DirectOneof {
    #[proto(tag = 200, validate = |v| v.cel([ &RULE ]))]
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

  pub fn random_cel_rule() -> CelRule {
    CelRule::builder()
      .id("hobbits")
      .message("they're taking the hobbits to isengard!")
      .expression("hobbits.location == isengard")
      .build()
  }

  static MSG_RULE: CachedProgram = cel_program!(random_cel_rule());

  static ABC: CachedRegex = regex!("abc", "abcde");
  static BYTES_REGEX: CachedBytesRegex = bytes_regex!("abc", "abcde");

  static AB: CachedList<&str> = cached_slice!(["abc"]);

  #[proto_message]
  #[proto(reserved_numbers(1, 2, 3..9))]
  #[proto(reserved_names("abc", "bcd"))]
  #[proto(nested_enums(PseudoEnum))]
  #[proto(nested_messages(Nested))]
  #[derive(Clone, Debug, Default)]
  #[proto(options = vec![ random_option() ])]
  pub struct Abc {
    #[proto(repeated(float), validate = |v| v.unique())]
    pub repeated_float: Vec<f32>,
    #[proto(timestamp, validate = |v| v.lt_now())]
    pub timestamp: Option<Timestamp>,

    #[proto(duration, validate = |v| v.lt(Duration { seconds: 2000, nanos: 0 }))]
    duration: Option<Duration>,

    #[proto(message(AbcProto, boxed), validate = |v| v.required())]
    boxed: Option<Box<Abc>>,

    #[proto(bytes, validate = |v| v.pattern(inline_bytes_regex!("abc", "abc")))]
    pub bytes: Bytes,

    #[proto(tag = 35, validate = |v| v.pattern(inline_regex!("abc", "abc")).in_(inline_cached_slice!(&str, ["abc"])))]
    name: String,

    #[proto(ignore, from_proto = Default::default)]
    num: Vec<i32>,

    #[proto(validate = |v| v.min_pairs(0).keys(|k| k.min_len(25)).values(|v| v.lt(25)))]
    map: HashMap<String, i32>,

    #[proto(map(string, enum_), validate = |v| v.min_pairs(20).values(|val| val.defined_only()))]
    enum_map: HashMap<String, PseudoEnum>,

    #[proto(map(string, message(proxied)), validate = |v| v.values(|val| val.ignore_always()))]
    message_map: HashMap<String, Nested>,

    #[proto(enum_, validate = |v| v.defined_only())]
    enum_field: PseudoEnum,

    #[proto(repeated(enum_), validate = |v| v.unique())]
    pub repeated_enum: Vec<PseudoEnum>,

    #[proto(enum_)]
    optional_enum: Option<PseudoEnum>,

    #[proto(message(proxied))]
    nested: Option<Nested>,

    #[proto(repeated(message(proxied)), validate = |v| v.min_items(1))]
    repeated_message: Vec<Nested>,

    #[proto(oneof(default, proxied))]
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
  #[derive(Clone, Debug)]
  pub struct Nested {
    name: String,
  }

  #[proto_message(direct)]
  pub struct Nested2 {
    name: String,

    num: i32,

    #[proto(oneof(proxied))]
    reused_oneof: Option<PseudoOneof>,
  }
}

use inner::*;
use protocheck::wrappers::Sint32;

fn main() {
  env_logger::init();

  let mut package = Package::new("abc");

  package.add_files([proto_file()]);

  package.check_unique_cel_rules().unwrap();
}

#[cfg(test)]
mod test {
  use std::collections::HashMap;

  use bytes::Bytes;
  use prelude::{Package, ProtoFile, ProtoMessage};
  use proc_macro_impls::proto_message;

  use crate::inner::{proto_file, PseudoEnum};

  #[proto_message(direct)]
  #[proto(package = "", file = "")]
  struct DuplicateRules {
    #[proto(tag = 1, validate = |v| v.cel([ inline_cel_program!(id = "abc", msg = "hi", expr = "hi"), inline_cel_program!(id = "abc", msg = "not hi", expr = "not hi") ]))]
    pub id: i32,
  }

  #[test]
  fn unique_rules() {
    let mut package = Package::new("abc");

    let mut file = ProtoFile::new("abc", "abc");

    file.add_messages([DuplicateRules::proto_schema()]);

    package.add_files([file]);

    assert!(package.check_unique_cel_rules().is_err());
  }

  #[proto_message(direct)]
  #[proto(package = "", file = "")]
  struct DummyMsg {
    #[proto(tag = 1)]
    pub id: i32,
  }

  #[proto_message(direct)]
  #[proto(package = "", file = "")]
  struct UniqueEnums {
    #[proto(repeated(enum_), tag = 1, validate = |v| v.unique())]
    pub unique_enums: Vec<PseudoEnum>,
  }

  #[test]
  fn unique_enums() {
    let mut msg = UniqueEnums {
      unique_enums: vec![PseudoEnum::AbcDeg as i32, PseudoEnum::AbcDeg as i32],
    };

    let err = msg.validate().unwrap_err();

    assert_eq!(err.first().unwrap().rule_id(), "repeated.unique");
  }

  #[proto_message(direct)]
  #[proto(package = "", file = "")]
  struct UniqueFloats {
    #[proto(tag = 1, validate = |v| v.unique())]
    pub unique_floats: Vec<f32>,
  }

  #[test]
  fn unique_floats() {
    let mut msg = UniqueFloats {
      unique_floats: vec![1.1, 1.1],
    };

    let err = msg.validate().unwrap_err();

    assert_eq!(err.first().unwrap().rule_id(), "repeated.unique");
  }

  #[proto_message(direct)]
  #[proto(package = "", file = "")]
  struct UniqueMessages {
    #[proto(repeated(message), tag = 1, validate = |v| v.unique())]
    pub unique_messages: Vec<DummyMsg>,
  }

  #[test]
  fn unique_messages() {
    let mut msg = UniqueMessages {
      unique_messages: vec![DummyMsg { id: 1 }, DummyMsg { id: 1 }],
    };

    let err = msg.validate().unwrap_err();

    assert_eq!(err.first().unwrap().rule_id(), "repeated.unique");
  }

  #[proto_message(direct)]
  #[proto(package = "", file = "")]
  struct UniqueBytes {
    #[proto(repeated(message), tag = 1, validate = |v| v.unique())]
    pub unique_bytes: Vec<Bytes>,
  }

  #[test]
  fn unique_bytes() {
    let mut msg = UniqueBytes {
      unique_bytes: vec![Bytes::default(), Bytes::default()],
    };

    let err = msg.validate().unwrap_err();

    assert_eq!(err.first().unwrap().rule_id(), "repeated.unique");
  }

  #[proto_message(direct)]
  #[proto(package = "", file = "")]
  struct MinItems {
    #[proto(repeated(int32), tag = 1, validate = |v| v.min_items(3))]
    pub items: Vec<i32>,
  }

  #[test]
  fn min_items() {
    let mut msg = MinItems { items: vec![] };

    let err = msg.validate().unwrap_err();

    assert_eq!(err.first().unwrap().rule_id(), "repeated.min_items");
  }

  #[proto_message(direct)]
  #[proto(package = "", file = "")]
  struct MaxItems {
    #[proto(repeated(int32), tag = 1, validate = |v| v.max_items(1))]
    pub items: Vec<i32>,
  }

  #[test]
  fn max_items() {
    let mut msg = MaxItems { items: vec![1, 2] };

    let err = msg.validate().unwrap_err();

    assert_eq!(err.first().unwrap().rule_id(), "repeated.max_items");
  }

  #[proto_message(direct)]
  #[proto(package = "", file = "")]
  struct MinPairs {
    #[proto(map(int32, int32), tag = 1, validate = |v| v.min_pairs(1))]
    pub items: HashMap<i32, i32>,
  }

  #[test]
  fn min_pairs() {
    let mut msg = MinPairs {
      items: HashMap::default(),
    };

    let err = msg.validate().unwrap_err();

    assert_eq!(err.first().unwrap().rule_id(), "map.min_pairs");
  }

  #[proto_message(direct)]
  #[proto(package = "", file = "")]
  struct MaxPairs {
    #[proto(map(int32, int32), tag = 1, validate = |v| v.max_pairs(1))]
    pub items: HashMap<i32, i32>,
  }

  #[test]
  fn max_pairs() {
    let mut map = HashMap::new();
    map.insert(1, 1);
    map.insert(2, 2);

    let mut msg = MaxPairs { items: map };

    let err = msg.validate().unwrap_err();

    assert_eq!(err.first().unwrap().rule_id(), "map.max_pairs");
  }
}
