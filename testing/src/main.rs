#![allow(unused)]

use std::collections::HashMap;

use prelude::{
  EnumValidator, GenericProtoEnum, IntValidator, OptionValue, ProtoOption, RepeatedValidator,
  RepeatedValidatorBuilder, Sint32, StringValidator, StringValidatorBuilder, ValidatorBuilderFor,
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

fn enum_validator() -> impl ValidatorBuilderFor<GenericProtoEnum> {
  let validator = EnumValidator::builder();

  validator.defined_only()
}

fn numeric_validator() -> impl ValidatorBuilderFor<Sint32> {
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
  use prelude::{cel_rule, CelRule, DEPRECATED};
  use proc_macro_impls::{
    proto_enum, proto_extension, proto_message, proto_oneof, proto_service, Extension, Service,
  };

  use super::*;

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
  enum PseudoEnum {
    AbcDeg,
    B,
    C,
  }

  #[proto_oneof]
  #[proto(required)]
  #[derive(Clone, Debug)]
  enum PseudoOneof {
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

  fn convert(map: HashMap<String, NestedProto>) -> HashMap<String, Nested> {
    map.into_iter().map(|(k, v)| (k, v.into())).collect()
  }

  fn message_rules() -> Vec<CelRule> {
    vec![
      CelRule::builder()
        .id("abc")
        .message("abc")
        .expression("abc")
        .build(),
      cel_rule!(id = "abc", msg = "abc", expr = "abc"),
    ]
  }

  #[proto_message]
  #[proto(reserved_numbers(1, 2, 3..9))]
  #[proto(reserved_names("abc", "bcd"))]
  #[proto(nested_enums(PseudoEnum))]
  #[proto(nested_messages(Nested))]
  #[derive(Clone, Debug, Default)]
  #[proto(options = vec![ random_option() ])]
  #[proto(validate = vec![ cel_rule!(id = "abc", msg = "abc", expr = "abc") ])]
  pub struct Abc {
    #[proto(timestamp, validate = |v| v.lt_now())]
    timestamp: Option<Timestamp>,

    #[proto(duration, validate = |v| v.lt(Duration { seconds: 2000, nanos: 0 }))]
    duration: Option<Duration>,

    #[proto(message(AbcProto, boxed))]
    boxed: Option<Box<Abc>>,

    #[proto(tag = 35, validate = string_validator())]
    name: Option<String>,

    #[proto(ignore, from_proto = Default::default)]
    num: Vec<i32>,

    #[proto(validate = |v| v.min_pairs(0).keys(|k| k.min_len(25)).values(|v| v.lt(25)))]
    map: HashMap<String, i32>,

    #[proto(map(string, enum_), validate = |v| v.values(|val| val.defined_only()))]
    enum_map: HashMap<String, PseudoEnum>,

    #[proto(map(string, message(proxied)), validate = |v| v.values(|val| val.ignore_always().cel(message_rules())))]
    message_map: HashMap<String, Nested>,

    #[proto(enum_, validate = enum_validator())]
    enum_field: PseudoEnum,

    #[proto(enum_)]
    optional_enum: Option<PseudoEnum>,

    #[proto(message(proxied))]
    nested: Option<Nested>,

    #[proto(repeated(message(proxied)))]
    repeated_message: Vec<Nested>,

    #[proto(oneof(default, proxied))]
    oneof: PseudoOneof,

    #[proto(sint32, validate = numeric_validator())]
    sint32: i32,

    #[proto(repeated(sint32), validate = |v| v.items(|it| it.gt(0)))]
    sint32_repeated: Vec<i32>,

    #[proto(map(sint32, sint32), validate = |v| v.keys(|k| k.gt(0)).values(|vals| vals.gt(0)))]
    sint32_map: HashMap<i32, i32>,

    #[proto(sint32)]
    sint32_optional: Option<i32>,
  }

  #[proto_message]
  #[derive(Clone, Debug)]
  pub struct Nested {
    name: String,
  }

  #[proto_message]
  #[proto(direct)]
  pub struct Nested2 {
    name: String,

    num: i32,

    #[proto(oneof(proxied))]
    reused_oneof: Option<PseudoOneof>,
  }
}

use inner::*;

fn main() {
  let mut file = prelude::ProtoFile::new("abc.proto", "myapp.v1");

  let mut file2 = proto_file();

  let mut msg = Abc::proto_schema();

  // let nested2 = Nested2::to_message();

  println!("{file2}");
  // let nested_enum = Bcd::to_nested_enum(nested);
}
