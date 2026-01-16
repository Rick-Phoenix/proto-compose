use super::*;

#[test]
fn recursive_message() {
  let valid_msg = BoxedMsg { msg: None, id: 1 };

  let mut msg = BoxedMsg {
    msg: Some(valid_msg.clone().into()),
    id: 1,
  };
  let baseline = msg.clone();

  assert!(msg.validate().is_ok(), "basic validation");

  macro_rules! assert_violation {
    ($violation:expr, $error:expr) => {
      assert_violation_id(&msg, $violation, $error);
      msg = baseline.clone();
    };
  }

  msg.id = 2;
  let invalid = msg.clone();

  assert_violation!("int32.const", "outer rule");

  msg.msg = Some(Box::new(invalid));
  assert_violation!("int32.const", "inner rule");
}

#[test]
fn direct_default_validator() {
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
