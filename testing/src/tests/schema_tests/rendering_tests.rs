#![allow(
  clippy::derive_partial_eq_without_eq,
  clippy::duplicated_attributes,
  clippy::struct_field_names
)]

use std::{collections::HashMap, fs, path::PathBuf};

use prelude::proto_types::{Duration, Timestamp};

use super::*;

proto_package!(RENDERING_PKG, name = "rendering");

define_proto_file!(
  RENDERING,
  name = "rendering.proto",
  package = RENDERING_PKG,
  options = test_options(),
  extensions = [TestExtension],
  imports = ["some_pkg/some_import.proto"]
);

#[test]
fn rendering_test() {
  let pkg = RENDERING_PKG.get_package();

  let output1 = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/proto_test"));

  pkg.render_files(&output1).unwrap();

  let manual_file = file_schema!(
    name = "rendering.proto",
    messages = [TestMessage = { messages = [Nested1 = { messages = [Nested2] }] }],
    extensions = [TestExtension],
    options = test_options(),
    services = [TestService],
    enums = [TestEnum],
    imports = ["some_pkg/some_import.proto"]
  );

  let manual_pkg = package_schema!("rendering", files = [manual_file]);
  let output2 = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/proto_test2"));

  manual_pkg.render_files(&output2).unwrap();

  let first_content = fs::read_to_string(output1.join("rendering.proto")).unwrap();
  let second_content = fs::read_to_string(output2.join("rendering.proto")).unwrap();

  assert_eq_pretty!(first_content, second_content);
}

#[test]
fn file_schema_output() {
  let pkg = RENDERING_PKG.get_package();

  let file = pkg.files.first().unwrap();

  assert_eq_pretty!(file.name, "rendering.proto");
  assert_eq_pretty!(file.package, "rendering");
  assert_eq_pretty!(file.options, test_options());
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

fn test_options() -> [ProtoOption; 2] {
  [normal_option(), nested_option()]
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

  let field1 = schema.fields.first().unwrap();

  assert_eq_pretty!(field1.name, "name");
  assert_eq_pretty!(field1.tag, 5000);
  assert_eq_pretty!(field1.options, test_options());
  assert_eq_pretty!(
    field1.type_,
    FieldType::Normal(ProtoType::Scalar(ProtoScalar::String))
  );

  let field2 = schema.fields.last().unwrap();

  assert_eq_pretty!(field2.name, "name2");
  assert_eq_pretty!(field2.tag, 5001);
  assert_eq_pretty!(
    field2.type_,
    FieldType::Normal(ProtoType::Scalar(ProtoScalar::String))
  );
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
  assert_eq_pretty!(service1.request.file, RENDERING.name);
  assert_eq_pretty!(service1.request.package, RENDERING.package);
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

#[proto_oneof]
#[proto(skip_checks(all))]
#[proto(options = test_options())]
pub enum OneofA {
  #[proto(tag = 201, options = test_options())]
  A(String),
  #[proto(fixed32, tag = 200, validate = |v| v.gt(10).lt(50))]
  B(u32),
}

#[test]
fn oneof_schema_output() {
  let schema = OneofA::proto_schema();

  assert_eq_pretty!(schema.options, test_options());

  let var1 = schema.fields.first().unwrap();
  assert_eq_pretty!(var1.options, test_options());

  let names_tags_and_types = [
    (
      "a",
      201,
      FieldType::Normal(ProtoType::Scalar(ProtoScalar::String)),
    ),
    (
      "b",
      200,
      FieldType::Normal(ProtoType::Scalar(ProtoScalar::Fixed32)),
    ),
  ];

  for (variant, (name, tag, type_)) in schema.fields.iter().zip(names_tags_and_types) {
    check_field(variant, tag, name, &type_);
  }
}

#[proto_oneof]
#[proto(skip_checks(all))]
pub enum OneofB {
  #[proto(tag = 1502)]
  A(String),
  #[proto(tag = 1503)]
  B(String),
}

fn msg_rule() -> CelProgram {
  cel_program!(id = "abc", msg = "abc", expr = "true == true")
}

#[proto_message]
#[proto(skip_checks(all))]
#[proto(reserved_numbers(1, 2, 3..9))]
#[proto(reserved_names("abc", "bcd"))]
#[proto(options = test_options())]
#[proto(validate = |v| v.cel(msg_rule()).cel(msg_rule()))]
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
fn check_field(field: &Field, tag: i32, name: &str, type_: &FieldType) {
  assert_eq_pretty!(field.tag, tag, "expected tag {tag} for field {name}");
  assert_eq_pretty!(field.name, name, "expected the field to be called {name}");
  assert_eq_pretty!(field.name, name, "expected the field to be called {name}");
  assert_eq_pretty!(
    &field.type_,
    type_,
    "expected field {name} to have type {type_:?}"
  );
}

#[test]
fn message_schema_output() {
  let schema = TestMessage::proto_schema();

  assert_eq_pretty!(schema.name, "TestMessage");
  assert_eq_pretty!(schema.options, test_options());

  let cel_rules: Vec<CelRule> = schema
    .validators
    .into_iter()
    .flat_map(|v| v.cel_rules)
    .collect();
  assert_eq_pretty!(cel_rules, &[msg_rule().rule, msg_rule().rule]);
  assert_eq_pretty!(schema.reserved_numbers, &[1..2, 2..3, 3..9]);
  assert_eq_pretty!(schema.reserved_names, &["abc", "bcd"]);

  assert_eq_pretty!(schema.entries.len(), 6);

  let fields = schema.entries.iter().filter_map(|e| e.as_field());
  let names_and_types = [
    (
      "manual_tag_field",
      FieldType::Normal(ProtoType::Scalar(ProtoScalar::Int32)),
    ),
    (
      "repeated_field",
      FieldType::Repeated(ProtoType::Scalar(ProtoScalar::Int32)),
    ),
    (
      "optional_field",
      FieldType::Optional(ProtoType::Scalar(ProtoScalar::Bytes)),
    ),
    (
      "msg_field",
      FieldType::Normal(ProtoType::Message(ProtoPath {
        name: "TestMessage.Nested1".into(),
        package: RENDERING_PKG.name.into(),
        file: RENDERING.name.into(),
      })),
    ),
    (
      "map_field",
      FieldType::Map {
        keys: ProtoMapKey::Sint32,
        values: ProtoType::Scalar(ProtoScalar::Sint32),
      },
    ),
  ];

  for (i, (field, (expected_name, expected_type))) in fields.zip(names_and_types).enumerate() {
    if i == 0 {
      check_field(field, 9, expected_name, &expected_type);
    } else {
      // First 9 numbers should be skipped because
      // 1-8 are reserved,
      // and 9 is manually occupied by the first field
      check_field(
        field,
        (i + 9).try_into().unwrap(),
        expected_name,
        &expected_type,
      );
    }
  }

  let oneof = schema
    .entries
    .iter()
    .find_map(|e| e.as_oneof())
    .unwrap();

  assert_eq_pretty!(
    oneof.validators.first().unwrap().clone(),
    ValidatorSchema {
      schema: ProtoOption {
        name: "(buf.validate.oneof).required".into(),
        value: true.into(),
      },
      cel_rules: vec![],
      imports: vec!["buf/validate/validate.proto".into()],
    },
    "oneof.required option should be present"
  )
}

#[proto_message]
#[proto(skip_checks(all))]
#[proto(parent_message = TestMessage)]
pub struct Nested1 {
  #[proto(oneof(tags(200, 201)), options = [proto_option!("some_option" => 100)])]
  reused_oneof: Option<OneofA>,
}

#[test]
fn reusable_oneofs() {
  let mut base_options = OneofA::proto_schema().options;

  let msg = Nested1::proto_schema();

  let MessageEntry::Oneof(oneof) = msg.entries.first().unwrap() else {
    panic!()
  };

  // The name of the oneof should correspond to that of the field
  assert_eq_pretty!(oneof.name, "reused_oneof");

  assert_ne!(
    base_options, oneof.options,
    "Extended and base options should not be the same"
  );

  // After adding the extra options, they should be the same
  base_options.push(proto_option!("some_option" => 100));
  assert_eq_pretty!(base_options, oneof.options);
}

#[proto_message]
#[proto(skip_checks(all))]
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
