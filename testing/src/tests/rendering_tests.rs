#![allow(
  clippy::derive_partial_eq_without_eq,
  clippy::duplicated_attributes,
  clippy::struct_field_names
)]

use std::sync::Arc;

use super::*;

define_proto_file!(
  RENDERING,
  file = "rendering.proto",
  package = TESTING_PKG,
  options = test_options()
);

#[test]
fn test_renders() {
  let pkg = prelude::collect_package("rendering_tests");

  pkg.render_files("proto_test").unwrap();
}

fn list_option() -> ProtoOption {
  ProtoOption {
    name: "list_option".into(),
    value: OptionValue::new_list([1i32, 2i32, 3i32, 4i32]),
  }
}

fn simple_option() -> ProtoOption {
  ProtoOption {
    name: "is_cool".into(),
    value: true.into(),
  }
}

fn message_option() -> ProtoOption {
  let mut values: Vec<(Arc<str>, OptionValue)> = Vec::new();

  values.push(("thing1".into(), 15.into()));
  values.push(("thing2".into(), OptionValue::Bool(true)));
  values.push(("thing3".into(), list_option().value));

  ProtoOption {
    name: "message_opt".into(),
    value: OptionValue::new_message(values),
  }
}

fn nested_message_option() -> ProtoOption {
  let mut values: Vec<(Arc<str>, OptionValue)> = Vec::new();

  values.push(("very_nested".into(), message_option().value));

  let mut outer: Vec<(Arc<str>, OptionValue)> = Vec::new();

  outer.push(("nested".into(), OptionValue::new_message(values)));

  ProtoOption {
    name: "nested_opt".into(),
    value: OptionValue::new_message(outer),
  }
}

fn test_options() -> Vec<ProtoOption> {
  vec![
    simple_option(),
    list_option(),
    message_option(),
    nested_message_option(),
  ]
}

#[proto_extension(target = MessageOptions)]
pub struct SomeExt {
  #[proto(options = test_options())]
  #[proto(tag = 5000)]
  name: String,

  #[proto(tag = 5001)]
  name2: String,
}

#[proto_service]
#[proto(options = test_options())]
pub enum FrodoService {
  #[proto(options = test_options())]
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
#[proto(options = test_options())]
pub enum TestEnum {
  #[proto(options = test_options())]
  AbcDeg,
  B,
}

#[proto_oneof]
#[proto(options = test_options())]
pub enum OneofA {
  #[proto(tag = 200, validate = |v| v.min_len(10).max_len(50))]
  A(String),
  #[proto(tag = 201)]
  B(i32),
}

#[proto_oneof]
pub enum OneofB {
  #[proto(tag = 1502)]
  A(String),
  #[proto(tag = 1503)]
  B(String),
}

fn msg_rule() -> CelProgram {
  cel_program!(id = "abc", msg = "abc", expr = "true == true")
}

#[proto_message(no_auto_test)]
#[proto(reserved_numbers(1, 2, 3..9))]
#[proto(reserved_names("abc", "bcd"))]
#[proto(options = test_options())]
#[proto(cel_rules(msg_rule(), msg_rule()))]
pub struct Abc {
  #[proto(repeated(int32), validate = |v| v.min_items(15).items(|it| it.gt(0).lt(50)))]
  pub repeated_field: Vec<i32>,

  #[proto(bytes, validate = |v| v.min_len(45).max_len(128))]
  pub optional_field: Option<Bytes>,

  #[proto(message, validate = |v| v.cel(cel_program!(id = "abc", msg = "abc", expr = "true == true")))]
  pub msg_field: Option<Nested>,

  #[proto(map(sint32, sint32), validate = |v| v.min_pairs(5).max_pairs(15).keys(|k| k.gt(15)).values(|vals| vals.gt(56)))]
  pub map_field: HashMap<i32, i32>,

  #[proto(oneof(required, tags(200, 201)))]
  pub oneof_field: Option<OneofA>,
}

#[proto_message(no_auto_test)]
#[proto(parent_message = Abc)]
pub struct Nested {
  #[proto(validate = |v| v.len_bytes(68))]
  name: String,
}

#[proto_message(no_auto_test)]
#[proto(parent_message = Nested)]
pub struct Nested2 {
  name: String,

  #[proto(map(sint32, sint32), validate = |v| v.min_pairs(5).max_pairs(15).keys(|k| k.gt(15)).values(|vals| vals.gt(56)))]
  pub map_field: HashMap<i32, i32>,

  #[proto(oneof(tags(200, 201)))]
  reused_oneof: Option<OneofA>,

  #[proto(oneof(tags(1502, 1503)))]
  oneof_b: Option<OneofB>,
}
