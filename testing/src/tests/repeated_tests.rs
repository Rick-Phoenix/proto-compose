use super::*;

#[proto_message(direct)]
#[proto(package = "", file = "")]
struct DummyMsg {
  #[proto(tag = 1)]
  pub id: i32,
}

#[proto_enum]
#[proto(package = "", file = "")]
enum DummyEnum {
  A,
  B,
  C,
}

#[proto_message(direct)]
#[proto(package = "", file = "")]
struct UniqueEnums {
  #[proto(repeated(enum_), tag = 1, validate = |v| v.unique())]
  pub unique_enums: Vec<DummyEnum>,
}

#[test]
fn unique_enums() {
  let mut msg = UniqueEnums {
    unique_enums: vec![DummyEnum::A as i32, DummyEnum::A as i32],
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
