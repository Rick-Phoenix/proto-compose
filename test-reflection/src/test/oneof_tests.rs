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
fn oneof_tests() {
  let mut msg = OneofTests {
    test_oneof: Some(TestOneof::String("a".to_string())),
  };
  let baseline = msg.clone();

  assert!(msg.validate().is_ok(), "basic validation");

  macro_rules! assert_violation {
    ($violation:expr, $error:expr) => {
      assert_violation_id(&msg, $violation, $error);
      msg = baseline.clone();
    };
  }

  msg.test_oneof = Some(TestOneof::BoxedMsg(Box::new(OneofTests {
    test_oneof: Some(TestOneof::String("b".to_string())),
  })));
  assert_violation!("string_cel_rule", "recursive oneof cel rule");

  msg.test_oneof = Some(TestOneof::BoxedMsg(Box::new(OneofTests {
    test_oneof: Some(TestOneof::String("c".to_string())),
  })));
  assert_violation!("recursive_cel_rule", "recursive oneof cel rule");

  msg.test_oneof = Some(TestOneof::DefaultValidatorMsg(DefaultValidatorTest {
    id: 2,
    test_oneof2: Some(TestOneof2::Number(1)),
    ..Default::default()
  }));
  assert_violation!("id_is_1", "default message validation");

  msg.test_oneof = Some(TestOneof::DefaultValidatorMsg(DefaultValidatorTest {
    id: 1,
    test_oneof2: Some(TestOneof2::Number(2)),
    ..Default::default()
  }));
  assert_violation!("int32.const", "default message validation");

  msg.test_oneof = Some(TestOneof::DefaultValidatorMsg(DefaultValidatorTest {
    id: 1,
    test_oneof2: Some(TestOneof2::Number(1)),
    ..Default::default()
  }));
  assert!(msg.validate().is_ok(), "default message validation");
  msg = baseline.clone();

  msg.test_oneof = Some(TestOneof::BoxedMsg(Box::new(msg.clone())));
  assert!(msg.validate().is_ok(), "recursive oneof validation");
}
