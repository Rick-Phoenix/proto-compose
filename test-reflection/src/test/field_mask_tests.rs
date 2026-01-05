use proto_types::FieldMask;

use crate::proto::FieldMaskRules;

use super::*;

fn test_value_1() -> FieldMask {
  FieldMask {
    paths: vec!["tom_bombadil".to_string()],
  }
}

fn test_value_2() -> FieldMask {
  FieldMask {
    paths: vec!["goldberry".to_string()],
  }
}

#[allow(unused)]
#[test]
fn field_mask_tests() {
  let mut msg = FieldMaskRules {
    const_test: Some(test_value_1()),
    in_test: Some(test_value_1()),
    not_in_test: Some(test_value_2()),
    required_test: Some(test_value_1()),
    ignore_always_test: Some(test_value_2()),
    cel_test: Some(test_value_1()),
  };

  let baseline = msg.clone();

  assert!(msg.validate().is_ok(), "basic validation");

  macro_rules! assert_violation {
    ($violation:expr, $error:literal) => {
      assert_violation_id(&msg, $violation, $error);
      msg = baseline.clone();
    };
  }

  msg.const_test = Some(test_value_2());
  assert_violation!("field_mask.const", "const rule");

  msg.in_test = Some(test_value_2());
  assert_violation!("field_mask.in", "in rule");

  msg.not_in_test = Some(test_value_1());
  assert_violation!("field_mask.not_in", "not_in rule");

  msg.cel_test = Some(test_value_2());
  assert_violation!("cel_rule", "cel rule");

  msg.required_test = None;
  assert_violation!("required", "required rule");
}
