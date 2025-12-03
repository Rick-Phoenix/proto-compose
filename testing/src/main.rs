#![allow(unused)]

use std::collections::HashMap;

use prelude::{
  validators::{
    EnumValidator, GenericProtoEnum, IntValidator, RepeatedValidator, RepeatedValidatorBuilder,
    Sint32, StringValidator, StringValidatorBuilder, ValidatorBuilderFor,
  },
  ProtoFile,
};
use proc_macro_impls::{Enum, Message, Oneof};
use proto_types::Duration;

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

#[proc_macro_impls::proto_module(file = "abc.proto", package = "myapp.v1")]
mod inner {
  use prelude::{
    validators::{ProtoValidator, ValidatorMap, *},
    *,
  };
  use proc_macro_impls::{proto_enum, proto_message, proto_oneof};

  use super::*;

  #[proto_enum]
  #[proto(reserved_numbers(1, 2, 10..))]
  #[derive(Clone, Debug)]
  enum PseudoEnum {
    AbcDeg,
    B,
    C,
  }

  #[proto_oneof]
  #[derive(Clone, Debug)]
  enum PseudoOneof {
    A(String),
    B(i32),
    #[proto(message(proxied, boxed))]
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
  #[proto(nested_enums(PseudoEnum))]
  #[derive(Clone, Debug, Default)]
  #[proto(validate = message_rules())]
  pub struct Abc {
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
  }
}

use inner::*;

fn main() {
  let mut file = ProtoFile::new("abc.proto", "myapp.v1");

  let mut msg = Abc::to_message();

  // let nested2 = Nested2::to_message();

  println!("{msg:#?}");
  // let nested_enum = Bcd::to_nested_enum(nested);
}
