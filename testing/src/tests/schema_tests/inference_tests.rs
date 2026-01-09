use super::*;

#[proto_message(no_auto_test)]
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

#[proto_message(proxied, no_auto_test)]
struct ProxiedEnumInference {
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

#[proto_message(no_auto_test)]
struct PrimitiveInference {
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

#[proto_message(no_auto_test)]
struct CardinalityInference {
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
#[proto_message(no_auto_test)]
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

#[proto_oneof(proxied, no_auto_test)]
enum ProxiedOneof {
  #[proto(tag = 1)]
  A(String),
  #[proto(tag = 2)]
  B(i32),
}

#[proto_message(proxied, no_auto_test)]
struct ProxiedMsg {
  id: i32,
}

#[proto_message(proxied, no_auto_test)]
struct ProxiedInference {
  #[proto(oneof(proxied, tags(1, 2)))]
  oneof: Option<ProxiedOneof>,
  #[proto(message(proxied))]
  msg: Option<ProxiedMsg>,
}

#[test]
fn proxied_inference() {
  let schema = ProxiedInferenceProto::proto_schema();

  let oneof = schema.entries.first().unwrap();

  assert_eq_pretty!(
    oneof,
    &MessageEntry::Oneof {
      oneof: ProxiedOneofProto::proto_schema(),
      required: false
    }
  );

  let msg_field = schema.entries.last().unwrap();

  assert_eq_pretty!(
    msg_field.as_field().unwrap().type_,
    FieldType::Normal(ProtoType::Message(ProxiedMsgProto::proto_path()))
  );
}
