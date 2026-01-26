use super::*;

#[proto_message]
#[proto(skip_checks(all))]
struct EnumInference {
  #[proto(enum_(TestEnum))]
  single_enum: i32,

  #[proto(repeated(enum_(TestEnum)))]
  repeated_enum: Vec<i32>,

  #[proto(optional(enum_(TestEnum)))]
  optional_enum: Option<i32>,

  #[proto(map(int32, enum_(TestEnum)))]
  enum_map: HashMap<i32, i32>,
}

#[test]
fn enum_inference() {
  let schema = EnumInference::proto_schema();

  let enum_type = ProtoType::Enum(TestEnum::proto_path());

  let exp_types = [
    FieldType::Normal(enum_type.clone()),
    FieldType::Repeated(enum_type.clone()),
    FieldType::Optional(enum_type.clone()),
    FieldType::Map {
      keys: ProtoMapKey::Int32,
      values: enum_type,
    },
  ];

  for (field, exp_type) in schema.fields().zip(exp_types) {
    assert_eq_pretty!(field.type_, exp_type);
  }
}

#[proto_message(proxied)]
#[proto(skip_checks(all))]
pub struct ProxiedEnumInference {
  #[proto(enum_)]
  single_enum: TestEnum,

  #[proto(repeated(enum_))]
  repeated_enum: Vec<TestEnum>,

  #[proto(optional(enum_))]
  optional_enum: Option<TestEnum>,

  #[proto(map(int32, enum_))]
  enum_map: HashMap<i32, TestEnum>,
}

#[test]
fn proxied_enum_inference() {
  let schema = ProxiedEnumInferenceProto::proto_schema();

  let enum_type = ProtoType::Enum(TestEnum::proto_path());

  let exp_types = [
    FieldType::Normal(enum_type.clone()),
    FieldType::Repeated(enum_type.clone()),
    FieldType::Optional(enum_type.clone()),
    FieldType::Map {
      keys: ProtoMapKey::Int32,
      values: enum_type,
    },
  ];

  for (field, exp_type) in schema.fields().zip(exp_types) {
    assert_eq_pretty!(field.type_, exp_type);
  }
}

#[proto_message]
#[proto(skip_checks(all))]
pub struct PrimitiveInference {
  int32: i32,
  int64: i64,
  boolean: bool,
  uint32: u32,
  uint64: u64,
  string: String,
  bytes: Bytes,
  f32: f32,
  f64: f64,
}

#[test]
fn primitive_inference() {
  let schema = PrimitiveInference::proto_schema();

  macro_rules! exp_types {
    ($($typ:ident),*) => {
      [
        $(
          FieldType::Normal(ProtoType::Scalar(ProtoScalar::$typ))
        ),*
      ]
    };
  }

  let expected_types = exp_types!(
    Int32, Int64, Bool, Uint32, Uint64, String, Bytes, Float, Double
  );

  for (field, exp_type) in schema.fields().zip(expected_types) {
    assert_eq_pretty!(field.type_, exp_type);
  }
}

#[proto_message]
#[proto(skip_checks(all))]
pub struct CardinalityInference {
  optional_inf: Option<String>,
  repeated_inf: Vec<String>,
  map_inf: HashMap<String, u64>,
}

#[test]
fn cardinality_inference() {
  let schema = CardinalityInference::proto_schema();

  let exp_types = [
    FieldType::Optional(ProtoType::Scalar(ProtoScalar::String)),
    FieldType::Repeated(ProtoType::Scalar(ProtoScalar::String)),
    FieldType::Map {
      keys: ProtoMapKey::String,
      values: ProtoType::Scalar(ProtoScalar::Uint64),
    },
  ];

  for (field, exp_type) in schema.fields().zip(exp_types) {
    assert_eq_pretty!(field.type_, exp_type);
  }
}

#[allow(clippy::use_self)]
#[proto_message]
#[proto(skip_checks(all))]
struct MessageInference {
  #[proto(message)]
  normal_inf: Option<CardinalityInference>,
  #[proto(repeated(message))]
  repeated_inf: Vec<CardinalityInference>,
  #[proto(map(int32, message))]
  map_inf: HashMap<i32, CardinalityInference>,
  #[proto(message)]
  boxed_inf: Option<Box<MessageInference>>,
}

#[test]
fn message_inference() {
  let schema = MessageInference::proto_schema();

  let msg_type = ProtoType::Message(CardinalityInference::proto_path());

  let exp_types = [
    FieldType::Normal(msg_type.clone()),
    FieldType::Repeated(msg_type.clone()),
    FieldType::Map {
      keys: ProtoMapKey::Int32,
      values: msg_type,
    },
    FieldType::Normal(ProtoType::Message(MessageInference::proto_path())),
  ];

  for (field, exp_type) in schema.fields().zip(exp_types) {
    assert_eq_pretty!(field.type_, exp_type);
  }
}

#[proto_oneof(proxied)]
#[proto(skip_checks(all))]
pub enum ProxiedOneof {
  #[proto(tag = 1)]
  A(String),
  #[proto(tag = 2)]
  B(i32),
}

#[proto_message(proxied)]
#[proto(skip_checks(all))]
pub struct ProxiedMsg {
  id: i32,
}

#[proto_message(proxied)]
#[proto(skip_checks(all))]
pub struct ProxiedInference {
  #[proto(oneof(proxied, tags(1, 2)))]
  oneof: Option<ProxiedOneof>,
  #[proto(message(proxied))]
  msg: Option<ProxiedMsg>,
}

#[test]
fn proxied_inference() {
  let schema = ProxiedInferenceProto::proto_schema();

  let MessageEntry::Oneof(oneof) = schema.entries.first().unwrap() else {
    panic!()
  };

  assert_eq_pretty!(
    oneof.name,
    "oneof",
    "The name should be overridden ('oneof', not 'proxied_oneof')"
  );

  let msg_field = schema.entries.last().unwrap();

  assert_eq_pretty!(
    msg_field.as_field().unwrap().type_,
    FieldType::Normal(ProtoType::Message(ProxiedMsgProto::proto_path()))
  );
}

#[proto_message]
#[proto(skip_checks(all))]
struct IntWrappers {
  #[proto(sint32)]
  sint32: i32,
  #[proto(sint64)]
  sint64: i64,
  #[proto(sfixed32)]
  sfixed32: i32,
  #[proto(sfixed64)]
  sfixed64: i64,
  #[proto(fixed32)]
  fixed32: u32,
  #[proto(fixed64)]
  fixed64: u64,
}

#[test]
fn int_wrappers() {
  let schema = IntWrappers::proto_schema();

  let exp_types = vec![
    ProtoType::Scalar(ProtoScalar::Sint32),
    ProtoType::Scalar(ProtoScalar::Sint64),
    ProtoType::Scalar(ProtoScalar::Sfixed32),
    ProtoType::Scalar(ProtoScalar::Sfixed64),
    ProtoType::Scalar(ProtoScalar::Fixed32),
    ProtoType::Scalar(ProtoScalar::Fixed64),
  ];

  for (field, exp_type) in schema.fields().zip(exp_types) {
    assert_eq_pretty!(field.type_, FieldType::Normal(exp_type))
  }
}

#[proto_message]
#[proto(skip_checks(all))]
struct OptionalIntWrappers {
  #[proto(sint32)]
  sint32: Option<i32>,
  #[proto(sint64)]
  sint64: Option<i64>,
  #[proto(sfixed32)]
  sfixed32: Option<i32>,
  #[proto(sfixed64)]
  sfixed64: Option<i64>,
  #[proto(fixed32)]
  fixed32: Option<u32>,
  #[proto(fixed64)]
  fixed64: Option<u64>,
}

#[test]
fn optional_int_wrappers() {
  let schema = OptionalIntWrappers::proto_schema();

  let exp_types = vec![
    ProtoType::Scalar(ProtoScalar::Sint32),
    ProtoType::Scalar(ProtoScalar::Sint64),
    ProtoType::Scalar(ProtoScalar::Sfixed32),
    ProtoType::Scalar(ProtoScalar::Sfixed64),
    ProtoType::Scalar(ProtoScalar::Fixed32),
    ProtoType::Scalar(ProtoScalar::Fixed64),
  ];

  for (field, exp_type) in schema.fields().zip(exp_types) {
    assert_eq_pretty!(field.type_, FieldType::Optional(exp_type))
  }
}

#[proto_message]
#[proto(skip_checks(all))]
struct RepeatedIntWrappers {
  #[proto(sint32)]
  sint32: Vec<i32>,
  #[proto(sint64)]
  sint64: Vec<i64>,
  #[proto(sfixed32)]
  sfixed32: Vec<i32>,
  #[proto(sfixed64)]
  sfixed64: Vec<i64>,
  #[proto(fixed32)]
  fixed32: Vec<u32>,
  #[proto(fixed64)]
  fixed64: Vec<u64>,
}

#[test]
fn repeated_int_wrappers() {
  let schema = RepeatedIntWrappers::proto_schema();

  let exp_types = vec![
    ProtoType::Scalar(ProtoScalar::Sint32),
    ProtoType::Scalar(ProtoScalar::Sint64),
    ProtoType::Scalar(ProtoScalar::Sfixed32),
    ProtoType::Scalar(ProtoScalar::Sfixed64),
    ProtoType::Scalar(ProtoScalar::Fixed32),
    ProtoType::Scalar(ProtoScalar::Fixed64),
  ];

  for (field, exp_type) in schema.fields().zip(exp_types) {
    assert_eq_pretty!(field.type_, FieldType::Repeated(exp_type))
  }
}

#[proto_message]
#[proto(skip_checks(all))]
struct MapIntWrappers {
  #[proto(map(sint32, sint32))]
  sint32: HashMap<i32, i32>,
  #[proto(map(sint64, sint64))]
  sint64: HashMap<i64, i64>,
  #[proto(map(sfixed32, sfixed32))]
  sfixed32: HashMap<i32, i32>,
  #[proto(map(sfixed64, sfixed64))]
  sfixed64: HashMap<i64, i64>,
  #[proto(map(fixed32, fixed32))]
  fixed32: HashMap<u32, u32>,
  #[proto(map(fixed64, fixed64))]
  fixed64: HashMap<u64, u64>,
}

#[test]
fn map_int_wrappers() {
  let schema = MapIntWrappers::proto_schema();

  macro_rules! create_exp_types {
    ($($types:ident),*) => {
      vec![
        $(
          FieldType::Map {
            keys: ProtoMapKey::$types,
            values: ProtoType::Scalar(ProtoScalar::$types)
          }
        ),*
      ]
    };
  }

  let exp_types = create_exp_types![Sint32, Sint64, Sfixed32, Sfixed64, Fixed32, Fixed64];

  for (field, exp_type) in schema.fields().zip(exp_types) {
    assert_eq_pretty!(field.type_, exp_type)
  }
}
