#![allow(
  clippy::derive_partial_eq_without_eq,
  clippy::duplicated_attributes,
  clippy::struct_field_names
)]

use std::collections::HashMap;

use prelude::proto_types::{Duration, Timestamp};

use super::*;

proto_package!(RENDERING_PKG, name = "rendering");

define_proto_file!(
  RENDERING,
  file = "rendering.proto",
  package = RENDERING_PKG,
  options = test_options(),
  extensions(TestExtension),
);

#[test]
fn file_schema_output() {
  let pkg = RENDERING_PKG.get_package();

  let file = pkg.files.first().unwrap();

  assert_eq_pretty!(file.name, "rendering.proto");
  assert_eq_pretty!(file.package, "rendering");
  assert_eq_pretty!(file.options, test_options());
}

#[test]
fn test_renders() {
  let pkg = RENDERING_PKG.get_package();

  let output = concat!(env!("CARGO_MANIFEST_DIR"), "/proto_test");

  pkg.render_files(output).unwrap();
}

fn test_option() -> OptionMessage {
  option_message!(
    "string" => "abc",
    "int" => 1,
    "float" => 1.0,
    "bytes" => b"hello",
    "num_list" => [1, 2, 3],
    "str_list" => ["abc", "def"],
    "bool" => true,
    "enum" => OptionValue::Enum("NAME".into()),
    "duration" => Duration::default(),
    "timestamp" => Timestamp::default()
  )
}

fn normal_option() -> ProtoOption {
  proto_option!("name" => test_option())
}

fn nested_option() -> ProtoOption {
  let innermost = option_message!("thing1" => 15,
    "thing2" => true,
    "thing3" => test_option()
  );

  let inner = option_message!("very_nested" => innermost);
  let outer = option_message!("nested" => inner);

  proto_option!("nested_opt" => outer)
}

fn test_options() -> Vec<ProtoOption> {
  vec![normal_option(), nested_option()]
}

#[proto_extension(target = MessageOptions)]
pub struct TestExtension {
  #[proto(options = test_options())]
  #[proto(tag = 5000)]
  name: String,

  #[proto(tag = 5001)]
  name2: String,
}

#[test]
fn extension_schema_output() {
  let schema = TestExtension::as_proto_extension();

  assert_eq_pretty!(schema.fields.len(), 2);

  assert_eq_pretty!(schema.fields[0].name, "name");
  assert_eq_pretty!(schema.fields[0].tag, 5000);
  assert_eq_pretty!(schema.fields[0].options, test_options());
  assert_eq_pretty!(schema.fields[1].name, "name2");
  assert_eq_pretty!(schema.fields[1].tag, 5001);
}

#[proto_service]
#[proto(options = test_options())]
pub enum TestService {
  #[proto(options = test_options())]
  Service1 {
    request: TestMessage,
    response: Nested1,
  },
  Service2 {
    request: TestMessage,
    response: Nested1,
  },
}

#[test]
fn service_schema_output() {
  let schema = TestService::as_proto_service();

  assert_eq_pretty!(schema.options, test_options());

  let service1 = schema.handlers.first().unwrap();

  assert_eq_pretty!(service1.name, "Service1");
  assert_eq_pretty!(service1.request.name, "TestMessage");
  assert_eq_pretty!(service1.response.name, "TestMessage.Nested1");
  assert_eq_pretty!(service1.options, test_options());
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

const PROTOBUF_MAX_TAG: i32 = 536_870_911;

#[test]
fn enum_schema_output() {
  let schema = TestEnum::proto_schema();

  assert_eq_pretty!(
    schema.reserved_numbers,
    &[1..2, 2..3, 10..PROTOBUF_MAX_TAG + 1]
  );
  assert_eq_pretty!(schema.reserved_names, &["abc", "bcd"]);
  assert_eq_pretty!(schema.options, test_options());

  let var1 = schema.variants.first().unwrap();

  assert_eq_pretty!(var1.name, "TEST_ENUM_ABC_DEG");
  assert_eq_pretty!(var1.options, test_options());
  assert_eq_pretty!(var1.tag, 0);

  let var2 = schema.variants.last().unwrap();

  assert_eq_pretty!(var2.name, "TEST_ENUM_B");
  // Should skip 1 and 2
  assert_eq_pretty!(var2.tag, 3);
}

#[proto_oneof(no_auto_test)]
#[proto(options = test_options())]
pub enum OneofA {
  #[proto(tag = 201, options = test_options())]
  A(String),
  #[proto(tag = 200, validate = |v| v.gt(10).lt(50))]
  B(i32),
}

#[test]
fn oneof_schema_output() {
  let schema = OneofA::proto_schema();

  assert_eq_pretty!(schema.options, test_options());

  let var1 = schema.fields.first().unwrap();

  assert_eq_pretty!(var1.tag, 201);
  assert_eq_pretty!(var1.options, test_options());
  assert_eq_pretty!(var1.name, "a");

  let var2 = schema.fields.last().unwrap();

  assert_eq_pretty!(var2.tag, 200);
  assert_eq_pretty!(var2.name, "b");
}

#[proto_oneof(no_auto_test)]
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
pub struct TestMessage {
  #[proto(tag = 9)]
  pub manual_tag_field: i32,

  #[proto(repeated(int32), validate = |v| v.min_items(15).items(|it| it.gt(0).lt(50)))]
  pub repeated_field: Vec<i32>,

  #[proto(bytes, validate = |v| v.min_len(45).max_len(128))]
  pub optional_field: Option<Bytes>,

  #[proto(message, validate = |v| v.cel(cel_program!(id = "abc", msg = "abc", expr = "true == true")))]
  pub msg_field: Option<Nested1>,

  #[proto(map(sint32, sint32), validate = |v| v.min_pairs(5).max_pairs(15).keys(|k| k.gt(15)).values(|vals| vals.gt(56)))]
  pub map_field: HashMap<i32, i32>,

  #[proto(oneof(required, tags(200, 201)))]
  pub oneof_field: Option<OneofA>,
}

#[track_caller]
fn check_tag_and_name(field: &Field, tag: i32, name: &str) {
  assert_eq_pretty!(field.tag, tag, "expected tag {tag} for field {name}");
  assert_eq_pretty!(field.name, name, "expected the field to be called {name}");
}

#[test]
fn message_schema_output() {
  let schema = TestMessage::proto_schema();

  assert_eq_pretty!(schema.name, "TestMessage");
  assert_eq_pretty!(schema.options, test_options());
  assert_eq_pretty!(schema.cel_rules, &[msg_rule().rule, msg_rule().rule]);
  assert_eq_pretty!(schema.reserved_numbers, &[1..2, 2..3, 3..9]);
  assert_eq_pretty!(schema.reserved_names, &["abc", "bcd"]);

  assert_eq_pretty!(schema.entries.len(), 6);

  let fields = schema.entries.iter().filter_map(|e| e.as_field());
  let names = [
    "manual_tag_field",
    "repeated_field",
    "optional_field",
    "msg_field",
    "map_field",
  ];

  for (i, (field, expected_name)) in fields.zip(names).enumerate() {
    if i == 0 {
      check_tag_and_name(field, 9, expected_name);
    } else {
      // First 9 numbers should be skipped because
      // 1-8 are reserved,
      // and 9 is manually occupied by the first field
      check_tag_and_name(field, (i + 9).try_into().unwrap(), expected_name);
    }
  }
}

#[proto_message(no_auto_test)]
#[proto(parent_message = TestMessage)]
pub struct Nested1 {
  #[proto(oneof(tags(200, 201)), options = [proto_option!("some_option" => 100)])]
  reused_oneof: Option<OneofA>,
}

#[test]
fn added_oneof_options() {
  let mut base_options = OneofA::proto_schema().options;

  let msg = Nested1::proto_schema();

  let MessageEntry::Oneof { oneof, .. } = msg.entries.first().unwrap() else {
    panic!()
  };

  assert_ne!(
    base_options, oneof.options,
    "Extended and base options should not be the same"
  );

  // After adding the extra options, they should be the same
  base_options.push(proto_option!("some_option" => 100));
  assert_eq_pretty!(base_options, oneof.options);
}

#[proto_message(no_auto_test)]
#[proto(parent_message = Nested1)]
pub struct Nested2 {
  name: String,

  #[proto(map(sint32, sint32), validate = |v| v.min_pairs(5).max_pairs(15).keys(|k| k.gt(15)).values(|vals| vals.gt(56)))]
  pub map_field: HashMap<i32, i32>,

  #[proto(oneof(tags(200, 201)))]
  reused_oneof: Option<OneofA>,

  #[proto(oneof(tags(1502, 1503)))]
  oneof_b: Option<OneofB>,
}
