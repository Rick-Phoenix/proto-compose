use super::*;

fn bad_rule() -> CelProgram {
  cel_program!(id = "abc", msg = "hi", expr = "hi")
}

#[proto_message(no_auto_test)]
struct BadFieldRules {
  #[proto(tag = 1, validate = |v| v.cel(bad_rule()))]
  pub id: i32,
}

#[test]
fn bad_field_rules() {
  let MessageTestError {
    message_full_name,
    field_errors,
    cel_errors,
  } = BadFieldRules::check_validators_consistency().unwrap_err();

  assert_eq_pretty!(message_full_name, "testing.BadFieldRules");
  assert_eq_pretty!(field_errors.len(), 1);
  // Top level rules, which don't apply here
  assert_eq_pretty!(cel_errors.len(), 0);
}

#[proto_message(no_auto_test)]
#[proto(cel_rules(bad_rule()))]
struct BadMsgRules {
  #[proto(tag = 1)]
  pub id: i32,
}

#[test]
fn bad_msg_rules() {
  let MessageTestError {
    message_full_name,
    field_errors,
    cel_errors,
  } = BadMsgRules::check_validators_consistency().unwrap_err();

  assert_eq_pretty!(message_full_name, "testing.BadMsgRules");
  assert_eq_pretty!(field_errors.len(), 0);
  assert_eq_pretty!(cel_errors.len(), 1);
}

#[allow(unused)]
#[proto_oneof(no_auto_test)]
enum BadCelOneof {
  #[proto(tag = 1, validate = |v| v.cel(bad_rule()))]
  Id(i32),
  #[proto(tag = 2)]
  Name(String),
}

#[test]
fn bad_oneof_rules() {
  let OneofErrors {
    oneof_name,
    field_errors: errors,
  } = BadCelOneof::check_validators_consistency().unwrap_err();

  assert_eq_pretty!(oneof_name, "BadCelOneof");
  assert_eq_pretty!(errors.len(), 1);
  assert!(matches!(errors[0].errors[0], ConsistencyError::CelError(_)));
}
