use super::*;

#[cfg(feature = "reflection")]
use crate::proto::oneof_tests::TestOneof;

#[cfg(not(feature = "reflection"))]
use test_schemas::DefaultValidatorRequiredOneof;

#[cfg(feature = "reflection")]
use crate::proto::DefaultValidatorRequiredOneof;

#[cfg(not(feature = "reflection"))]
use test_schemas::TestOneof;

#[cfg(feature = "reflection")]
use crate::proto::default_validator_test_oneof::DefaultValidatorOneof;

#[cfg(not(feature = "reflection"))]
use test_schemas::ValidatorRequiredOneof;

#[cfg(feature = "reflection")]
use crate::proto::default_validator_required_oneof::ValidatorRequiredOneof;

#[test]
fn required_oneof_validation() {
  let mut msg = DefaultValidatorRequiredOneof {
    validator_required_oneof: Some(ValidatorRequiredOneof::A(1)),
  };

  assert!(msg.validate().is_ok());

  msg.validator_required_oneof = None;

  assert_violation_id(
    &msg,
    "oneof.required",
    "required oneof should trigger validation",
  );
}

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

  msg.test_oneof = Some(TestOneof::BoxedMsg(Box::new(msg.clone())));
  assert!(msg.validate().is_ok(), "recursive oneof validation");
}

#[test]
fn default_validator_oneof() {
  let mut msg = DefaultValidatorTestOneof {
    default_validator_oneof: Some(DefaultValidatorOneof::A(SimpleMsg {
      id: 1,
      name: "abc".to_string(),
    })),
  };

  assert!(msg.is_valid(), "basic validation");

  msg.default_validator_oneof = Some(DefaultValidatorOneof::A(SimpleMsg {
    id: 2,
    name: "abc".to_string(),
  }));

  assert!(
    !msg.is_valid(),
    "default validator should be registered for a oneof variant if it's a message"
  );
}

#[test]
fn required_oneof() {
  let mut msg = DefaultValidatorTestOneof {
    default_validator_oneof: Some(DefaultValidatorOneof::A(SimpleMsg {
      id: 1,
      name: "abc".to_string(),
    })),
  };

  assert!(msg.is_valid(), "basic validation");

  msg.default_validator_oneof = None;

  assert_violation_id(&msg, "oneof.required", "oneof required");
}
