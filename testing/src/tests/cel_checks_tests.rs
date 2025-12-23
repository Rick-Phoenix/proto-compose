use super::*;

static RULE_1: CachedProgram = cel_program!(id = "abc", msg = "hi", expr = "true == true");
static RULE_2: CachedProgram = cel_program!(id = "abc", msg = "not hi", expr = "false == false");

#[proto_message(direct)]
struct FieldDuplicateRules {
  #[proto(tag = 1, validate = |v| v.cel(&RULE_1).cel(&RULE_2))]
  pub id: i32,
}

#[test]
fn field_duplicate_rules() {
  let mut file = ProtoFile::new("abc", "abc");

  file.add_messages([FieldDuplicateRules::proto_schema()]);

  let package = Package {
    name: "abc",
    files: vec![file],
  };

  assert!(package.check_unique_cel_rules().is_err());
}

#[proto_message(direct)]
#[proto(cel_rules(RULE_1, RULE_2))]
struct MsgDuplicateRules {
  #[proto(tag = 1)]
  pub id: i32,
}

#[test]
fn msg_duplicate_rules() {
  let mut file = ProtoFile::new("abc", "abc");

  file.add_messages([MsgDuplicateRules::proto_schema()]);

  let package = Package {
    name: "abc",
    files: vec![file],
  };

  assert!(package.check_unique_cel_rules().is_err());
}

#[proto_message(direct)]
#[proto(cel_rules(RULE_1))]
struct MsgAndFieldDuplicateRules {
  #[proto(tag = 1, validate = |v| v.cel(&RULE_2))]
  pub id: i32,
}

#[test]
fn msg_and_field_duplicate_rules() {
  let mut file = ProtoFile::new("abc", "abc");

  file.add_messages([MsgAndFieldDuplicateRules::proto_schema()]);

  let package = Package {
    name: "abc",
    files: vec![file],
  };

  assert!(package.check_unique_cel_rules().is_err());
}

#[proto_oneof(direct)]
enum OneofWithRule {
  #[proto(tag = 1, validate = |v| v.cel(&RULE_1))]
  Id(i32),
  #[proto(tag = 2)]
  Name(String),
}

#[proto_message(direct)]
#[proto(cel_rules(RULE_2))]
struct MsgAndOneofDuplicateRules {
  #[proto(oneof(tags(1, 2)))]
  pub oneof: Option<OneofWithRule>,
}

#[test]
fn msg_and_oneof_duplicate_rules() {
  let mut file = ProtoFile::new("abc", "abc");

  file.add_messages([MsgAndOneofDuplicateRules::proto_schema()]);

  let package = Package {
    name: "abc",
    files: vec![file],
  };

  assert!(package.check_unique_cel_rules().is_err());
}

#[proto_message(direct)]
struct FieldAndOneofDuplicateRules {
  #[proto(oneof(tags(1, 2)))]
  pub oneof: Option<OneofWithRule>,
  #[proto(validate = |v| v.cel(&RULE_2))]
  pub id: i32,
}

#[test]
fn field_and_oneof_duplicate_rules() {
  let mut file = ProtoFile::new("abc", "abc");

  file.add_messages([FieldAndOneofDuplicateRules::proto_schema()]);

  let package = Package {
    name: "abc",
    files: vec![file],
  };

  assert!(package.check_unique_cel_rules().is_err());
}

#[proto_oneof(direct)]
enum DuplicateRuleOneof {
  #[proto(tag = 1, validate = |v| v.cel(&RULE_1))]
  Id(i32),
  #[proto(tag = 2, validate = |v| v.cel(&RULE_2))]
  Name(String),
}

#[proto_message(direct)]
struct OneofDuplicateRules {
  #[proto(oneof(tags(1, 2)))]
  pub oneof: Option<DuplicateRuleOneof>,
}

#[test]
fn oneof_duplicate_rules() {
  let mut file = ProtoFile::new("abc", "abc");

  file.add_messages([OneofDuplicateRules::proto_schema()]);

  let package = Package {
    name: "abc",
    files: vec![file],
  };

  assert!(package.check_unique_cel_rules().is_err());
}

// This one should be okay because it's the same rule used twice, not
// two different rules with the same ID
#[proto_message(direct)]
#[proto(cel_rules(RULE_1))]
struct BenignDuplicateRules {
  #[proto(tag = 1, validate = |v| v.cel(&RULE_1))]
  pub id: i32,
}

#[test]
fn benign_duplicate_rules() {
  let mut file = ProtoFile::new("abc", "abc");

  file.add_messages([BenignDuplicateRules::proto_schema()]);

  let package = Package {
    name: "abc",
    files: vec![file],
  };

  assert!(package.check_unique_cel_rules().is_ok());
}

static BAD_RULE: CachedProgram = cel_program!(id = "abc", msg = "hi", expr = "hi");

#[proto_message(direct, no_auto_test)]
struct BadFieldRules {
  #[proto(tag = 1, validate = |v| v.cel(&BAD_RULE))]
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

#[proto_message(direct, no_auto_test)]
#[proto(cel_rules(BAD_RULE))]
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

#[proto_oneof(direct)]
enum BadCelOneof {
  #[proto(tag = 1, validate = |v| v.cel(&BAD_RULE))]
  Id(i32),
  #[proto(tag = 2)]
  Name(String),
}

#[proto_message(direct, no_auto_test)]
struct BadOneofRules {
  #[proto(oneof(tags(1, 2)))]
  pub oneof: Option<BadCelOneof>,
}

#[test]
fn bad_oneof_rules() {
  let MessageTestError {
    message_full_name,
    field_errors,
    cel_errors,
  } = BadOneofRules::check_validators_consistency().unwrap_err();

  assert_eq_pretty!(message_full_name, "testing.BadOneofRules");
  assert_eq_pretty!(field_errors.len(), 1);
  // Top level rules, which don't apply here
  assert_eq_pretty!(cel_errors.len(), 0);
}
