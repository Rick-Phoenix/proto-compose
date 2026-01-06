use bytes::Bytes;

use super::*;

#[test]
fn repeated_tests() {
  let mut msg = RepeatedItemsTests {
    items_test: vec![15, 15],
    items_cel_test: vec![1],
    cel_test: vec![1],
  };
  let baseline = msg.clone();

  assert!(msg.validate().is_ok(), "basic validation");

  macro_rules! assert_violation {
    ($violation:expr, $error:expr) => {
      assert_violation_id(&msg, $violation, $error);
      msg = baseline.clone();
    };
  }

  macro_rules! assert_violation_path {
    ($violation:expr, $error:literal) => {
      assert_eq!(full_rule_id_path(&msg), $violation, $error);
      msg = baseline.clone();
    };
  }

  msg.items_test = vec![11, 15];
  assert_violation_path!("repeated.items.int32.const", "items rule");

  msg.items_cel_test = vec![2];
  assert_violation_path!("repeated.items.cel", "items cel rule");

  msg.cel_test = vec![2];
  assert_violation!("cel_rule", "cel rule");

  msg.cel_test = vec![];
  assert!(msg.validate().is_ok(), "Should ignore empty vec");
}

#[test]
fn unique_enums() {
  let mut msg = UniqueEnums {
    unique_enums: vec![DummyEnum::A as i32, DummyEnum::A as i32],
  };

  let err = msg.validate().unwrap_err();

  assert_eq!(err.first().unwrap().rule_id(), "repeated.unique");

  msg.unique_enums = vec![DummyEnum::A as i32, DummyEnum::B as i32];
  assert!(msg.validate().is_ok());
}

#[test]
fn unique_floats() {
  let mut msg = UniqueFloats {
    unique_floats: vec![1.1, 1.1],
  };

  let err = msg.validate().unwrap_err();

  assert_eq!(err.first().unwrap().rule_id(), "repeated.unique");

  msg.unique_floats = vec![1.5, 2.5];
  assert!(msg.validate().is_ok());
}

#[test]
fn unique_messages() {
  let mut msg = UniqueMessages {
    unique_messages: vec![DummyMsg { id: 1 }, DummyMsg { id: 1 }],
  };

  let err = msg.validate().unwrap_err();

  assert_eq!(err.first().unwrap().rule_id(), "repeated.unique");

  msg.unique_messages = vec![DummyMsg { id: 1 }, DummyMsg { id: 2 }];
  assert!(msg.validate().is_ok());
}

#[test]
fn unique_bytes() {
  let mut msg = UniqueBytes {
    unique_bytes: vec![Bytes::default(), Bytes::default()],
  };

  let err = msg.validate().unwrap_err();

  assert_eq!(err.first().unwrap().rule_id(), "repeated.unique");

  msg.unique_bytes = vec![Bytes::default(), Bytes::from_static(b"hi")];
  assert!(msg.validate().is_ok());
}

#[test]
fn min_items() {
  let mut msg = MinItems { items: vec![] };

  let err = msg.validate().unwrap_err();

  assert_eq!(err.first().unwrap().rule_id(), "repeated.min_items");

  msg.items = vec![1, 2, 3];
  assert!(msg.validate().is_ok());
}

#[test]
fn max_items() {
  let mut msg = MaxItems { items: vec![1, 2] };

  let err = msg.validate().unwrap_err();

  assert_eq!(err.first().unwrap().rule_id(), "repeated.max_items");

  msg.items = vec![1];
  assert!(msg.validate().is_ok());
}
