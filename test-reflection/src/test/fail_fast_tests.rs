use bytes::Bytes;
use proto_types::FieldMask;

#[cfg(feature = "reflection")]
use crate::proto::fail_fast_test::SimpleOneof;

#[cfg(not(feature = "reflection"))]
use test_schemas::SimpleOneof;

use super::*;

// Checks if each validator exits early
#[test]
fn validators_fail_fast_test() {
  let mut msg = FailFastTest {
    string: "a".to_string(),
    bytes: Bytes::from_static(b"a"),
    int: 2,
    float: 2.0,
    duration: Some(Duration::new(1, 0)),
    timestamp: Some(Timestamp::now() + Duration::new(5, 0)),
    field_mask: Some(FieldMask {
      paths: vec!["abc".to_string()],
    }),
    enum_field: TestEnum::One.into(),
    simple_oneof: Some(SimpleOneof::A(1)),
    message: Some(SimpleMsg {
      id: 1,
      name: "abc".to_string(),
    }),
  };

  let baseline = msg.clone();

  assert!(msg.is_valid(), "basic validation");

  macro_rules! assert_violation {
    () => {
      assert!(msg.validate().unwrap_err().violations.len() == 1);
      // The `validate_all` method should not exit early
      assert!(msg.validate_all().unwrap_err().len() != 1);
      msg = baseline.clone();
    };
  }

  msg.string = "abc".to_string();
  assert_violation!();

  msg.bytes = Bytes::from_static(b"abc");
  assert_violation!();

  msg.int = 1;
  assert_violation!();

  msg.float = 1.0;
  assert_violation!();

  msg.duration = Some(Duration::default());
  assert_violation!();

  msg.timestamp = Some(Timestamp::default());
  assert_violation!();

  msg.field_mask = Some(FieldMask {
    paths: vec!["abcde".to_string()],
  });
  assert_violation!();

  msg.enum_field = 45;
  assert_violation!();

  msg.message = Some(SimpleMsg {
    id: 2,
    name: "a".to_string(),
  });
  assert_violation!();
}

// Checks if a message validator exits early
#[test]
fn message_fail_fast_test() {
  let mut msg = FailFastTest {
    string: "a".to_string(),
    bytes: Bytes::from_static(b"a"),
    int: 2,
    float: 2.0,
    duration: Some(Duration::new(1, 0)),
    timestamp: Some(Timestamp::now() + Duration::new(5, 0)),
    field_mask: Some(FieldMask {
      paths: vec!["abc".to_string()],
    }),
    enum_field: TestEnum::One.into(),
    simple_oneof: Some(SimpleOneof::A(1)),
    message: Some(SimpleMsg {
      id: 1,
      name: "abc".to_string(),
    }),
  };

  assert!(msg.is_valid(), "basic validation");

  msg.string = "abc".to_string();
  msg.bytes = Bytes::from_static(b"abc");
  msg.int = 1;
  msg.float = 1.0;
  msg.duration = Some(Duration::default());
  msg.timestamp = Some(Timestamp::default());
  msg.field_mask = Some(FieldMask {
    paths: vec!["abcde".to_string()],
  });
  msg.enum_field = 45;
  msg.message = Some(SimpleMsg {
    id: 2,
    name: "a".to_string(),
  });

  assert!(msg.validate().unwrap_err().violations.len() == 1);

  // The `validate_all` method should not exit early
  assert!(msg.validate_all().unwrap_err().len() != 1);
}
