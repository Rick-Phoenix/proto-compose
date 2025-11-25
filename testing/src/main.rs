#![allow(unused)]

use std::collections::HashMap;

use prelude::{
  validators::{
    RepeatedValidator, RepeatedValidatorBuilder, StringValidator, StringValidatorBuilder,
    ValidatorBuilderFor,
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

#[proc_macro_impls::proto_module(file = "abc.proto", package = "myapp.v1")]
mod inner {
  use prelude::{
    validators::{ProtoValidator, ValidatorMap, *},
    *,
  };
  use proc_macro_impls::proto_enum;

  use super::*;

  #[proto_enum]
  #[proto(reserved_numbers(1, 2, 10..))]
  enum PseudoEnum {
    AbcDeg,
    B,
    C,
  }

  #[derive(Oneof)]
  #[proto(required)]
  enum PseudoOneof {
    #[proto(tag = 12, validate = |v| v)]
    A(String),
    B(i32),
  }

  #[derive(Message)]
  #[proto(reserved_numbers(1, 2, 3..9))]
  #[proto(nested_messages(Nested))]
  #[proto(nested_enums(PseudoEnum))]
  pub struct Abc {
    #[proto(tag = 35, validate = string_validator())]
    name: Option<String>,

    #[proto(validate = repeated_validator())]
    num: Vec<i32>,

    #[proto(validate = |v| v.min_pairs(0).keys(|k| k.min_len(25)).values(|v| v.lt(25)))]
    map: HashMap<String, i32>,

    #[proto(oneof)]
    oneof: Option<PseudoOneof>,

    #[proto(enum_, validate = |v| v.defined_only())]
    enum_field: PseudoEnum,
  }

  #[derive(Message)]
  #[proto(nested_messages(Nested2))]
  pub struct Nested {
    name: String,
  }

  #[derive(Message)]
  pub struct Nested2 {
    name: String,

    #[proto(validate = |v| v.ignore_always())]
    nested1: Nested,
  }
}

use inner::*;

fn main() {
  let mut file = ProtoFile::new("abc.proto", "myapp.v1");

  let mut msg = Abc::to_message();

  let nested2 = Nested2::to_message();

  println!("{msg:#?}");
  // let nested_enum = Bcd::to_nested_enum(nested);
}
