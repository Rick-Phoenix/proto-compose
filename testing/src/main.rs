#![allow(unused)]

use std::collections::HashMap;

use prelude::{
  validators::{
    EnumValidator, GenericProtoEnum, RepeatedValidator, RepeatedValidatorBuilder, StringValidator,
    StringValidatorBuilder, ValidatorBuilderFor,
  },
  ProtoFile,
};
use proc_macro_impls::{Enum, Message, Oneof};

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
  #[derive(Clone, Debug, prost::Enumeration)]
  enum PseudoEnum {
    AbcDeg,
    B,
    C,
  }

  // #[proto(required)]
  #[proto_oneof]
  #[proto(direct)]
  enum PseudoOneof {
    // #[proto(tag = 12, validate = |v| v)]
    A(String),
    B(i32),
  }

  fn convert(map: HashMap<String, NestedProto>) -> HashMap<String, Nested> {
    map.into_iter().map(|(k, v)| (k, v.into())).collect()
  }

  #[proto_message]
  #[proto(reserved_numbers(1, 2, 3..9))]
  #[proto(nested_enums(PseudoEnum))]
  #[derive(Clone, Debug, Default)]
  pub struct Abc {
    #[proto(message(AbcProto))]
    boxed: Option<Box<Abc>>,

    #[proto(tag = 35, validate = string_validator())]
    name: Option<String>,

    #[proto(ignore, validate = repeated_validator())]
    num: Vec<i32>,

    #[proto(validate = |v| v.min_pairs(0).keys(|k| k.min_len(25)).values(|v| v.lt(25)))]
    map: HashMap<String, i32>,

    #[proto(map(string, enum_), validate = |v| v.values(|val| val.defined_only()))]
    enum_map: HashMap<String, PseudoEnum>,

    #[proto(map(string, message(suffixed)), validate = |v| v.values(|val| val.ignore_always()))]
    message_map: HashMap<String, Nested>,

    #[proto(enum_, validate = enum_validator())]
    enum_field: PseudoEnum,

    #[proto(message(suffixed), from_proto = |v| v.map(Into::into))]
    nested: Option<Nested>,

    #[proto(oneof)]
    oneof: Option<PseudoOneof>,
  }

  fn from_proto(input: NestedProto) -> Nested {
    Nested { name: input.name }
  }

  fn into_proto(input: Nested) -> NestedProto {
    NestedProto { name: input.name }
  }

  #[proto_message]
  #[proto(from_proto = from_proto)]
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
