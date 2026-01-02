use super::*;

fn rule_1() -> CelProgram {
  cel_program!(id = "abc", msg = "hi", expr = "true == true")
}

fn rule_2() -> CelProgram {
  cel_program!(id = "abc", msg = "not hi", expr = "false == false")
}

#[proto_message(no_auto_test)]
struct FieldDuplicateRules {
  #[proto(tag = 1, validate = |v| v.cel(rule_1()).cel(rule_2()))]
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

#[proto_message(no_auto_test)]
#[proto(cel_rules(rule_1(), rule_2()))]
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

#[proto_message(no_auto_test)]
#[proto(cel_rules(rule_1()))]
struct MsgAndFieldDuplicateRules {
  #[proto(tag = 1, validate = |v| v.cel(rule_2()))]
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

#[proto_oneof]
enum OneofWithRule {
  #[proto(tag = 1, validate = |v| v.cel(rule_1()))]
  Id(i32),
  #[proto(tag = 2)]
  Name(String),
}

#[proto_message(no_auto_test)]
#[proto(cel_rules(rule_2()))]
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

#[proto_message(no_auto_test)]
struct FieldAndOneofDuplicateRules {
  #[proto(oneof(tags(1, 2)))]
  pub oneof: Option<OneofWithRule>,
  #[proto(validate = |v| v.cel(rule_2()))]
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

#[proto_oneof]
enum DuplicateRuleOneof {
  #[proto(tag = 1, validate = |v| v.cel(rule_1()))]
  Id(i32),
  #[proto(tag = 2, validate = |v| v.cel(rule_2()))]
  Name(String),
}

#[proto_message(no_auto_test)]
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
#[proto_message(no_auto_test)]
#[proto(cel_rules(rule_1()))]
struct BenignDuplicateRules {
  #[proto(tag = 1, validate = |v| v.cel(rule_1()))]
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
