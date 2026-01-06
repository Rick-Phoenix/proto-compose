#[cfg(feature = "reflection")]
use crate::proto::default_validator_test::TestOneof2;
#[cfg(not(feature = "reflection"))]
use test_schemas::TestOneof2;

#[cfg(feature = "reflection")]
use crate::proto::oneof_tests::TestOneof;

#[cfg(not(feature = "reflection"))]
use test_schemas::TestOneof;

use super::*;

#[test]
fn message_tests() {
  let mut msg = DefaultValidatorTest2 {
    msg_with_default_validator: Some(DefaultValidatorTest {
      id: 1,
      test_oneof2: Some(TestOneof2::Number(1)),
      ..Default::default()
    }),
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

  let invalid = DefaultValidatorTest {
    id: 2,
    test_oneof2: Some(TestOneof2::Number(1)),
    ..Default::default()
  };

  msg.msg_with_default_validator = Some(invalid.clone());
  assert_violation!("id_is_1", "cel rule");

  msg.msg_with_default_validator = Some(DefaultValidatorTest {
    id: 1,
    repeated_test: vec![invalid.clone()],
    test_oneof2: Some(TestOneof2::Number(1)),
    ..Default::default()
  });
  assert_violation_path!("repeated.items.cel", "default repeated validator cel rule");

  msg.msg_with_default_validator = Some(DefaultValidatorTest {
    id: 1,
    map_test: hashmap! { 1 => invalid.clone() },
    test_oneof2: Some(TestOneof2::Number(1)),
    ..Default::default()
  });
  assert_violation_path!("map.values.cel", "default map validator cel rule");
}
