use bytes::Bytes;

use super::*;

// Verifies if validators exit after checking `const` even if other rules are defined
#[test]
fn const_rules_test() {
  let mut msg = ConstRulesTest {
    string: "abc".to_string(),
    bytes: Bytes::from_static(b"abc"),
    int: 3,
    float: 3.0,
    duration: Some(Duration::new(1, 0)),
    timestamp: Some(Timestamp::new(1, 0)),
    field_mask: Some(FieldMask {
      paths: vec!["abc".to_string()],
    }),
    enum_field: TestEnum::One.into(),
  };

  let baseline = msg.clone();

  assert!(msg.is_valid(), "basic validation");

  macro_rules! assert_violation {
    ($violation:expr) => {
      assert_violation_id(
        &msg,
        concat!($violation, ".const"),
        concat!($violation, ".const"),
      );
      msg = baseline.clone();
    };
  }

  msg.string = "a".to_string();
  assert_violation!("string");

  msg.bytes = Bytes::from_static(b"a");
  assert_violation!("bytes");

  msg.int = 0;
  assert_violation!("int32");

  msg.float = 0.0;
  assert_violation!("float");

  msg.duration = Some(Duration::default());
  assert_violation!("duration");

  msg.timestamp = Some(Timestamp::default());
  assert_violation!("timestamp");

  msg.field_mask = Some(FieldMask {
    paths: vec!["a".to_string()],
  });
  assert_violation!("field_mask");

  msg.enum_field = 100;
  assert_violation!("enum");
}
