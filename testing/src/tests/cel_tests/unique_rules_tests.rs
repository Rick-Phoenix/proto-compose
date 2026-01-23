use super::*;

fn prog_1() -> CelProgram {
  cel_program!(id = "abc", msg = "hi", expr = "true == true")
}

fn prog_2() -> CelProgram {
  cel_program!(id = "abc", msg = "not hi", expr = "false == false")
}

#[proto_message]
#[proto(skip_checks(all))]
struct FieldDuplicateRules {
  #[proto(tag = 1, validate = |v| v.cel(prog_1()).cel(prog_2()))]
  pub id: i32,
}

#[test]
fn field_duplicate_rules() {
  let mut file = ProtoFile::new("abc", "abc");

  file.with_messages([FieldDuplicateRules::proto_schema()]);

  let package = Package {
    name: "abc".into(),
    files: vec![file],
  };

  assert!(package.check_unique_cel_rules().is_err());
}

#[proto_message]
#[proto(skip_checks(all))]
#[proto(validate = |v| v.cel(prog_1()).cel(prog_2()))]
struct MsgDuplicateRules {
  #[proto(tag = 1)]
  pub id: i32,
}

#[test]
fn msg_duplicate_rules() {
  let mut file = ProtoFile::new("abc", "abc");

  file.with_messages([MsgDuplicateRules::proto_schema()]);

  let package = Package {
    name: "abc".into(),
    files: vec![file],
  };

  assert!(package.check_unique_cel_rules().is_err());
}

#[proto_message]
#[proto(skip_checks(all))]
#[proto(validate = |v| v.cel(prog_1()))]
struct MsgAndFieldDuplicateRules {
  #[proto(tag = 1, validate = |v| v.cel(prog_2()))]
  pub id: i32,
}

#[test]
fn msg_and_field_duplicate_rules() {
  let mut file = ProtoFile::new("abc", "abc");

  file.with_messages([MsgAndFieldDuplicateRules::proto_schema()]);

  let package = Package {
    name: "abc".into(),
    files: vec![file],
  };

  assert!(package.check_unique_cel_rules().is_err());
}

#[proto_oneof]
#[proto(skip_checks(all))]
enum OneofWithRule {
  #[proto(tag = 1, validate = |v| v.cel(prog_1()))]
  Id(i32),
  #[proto(tag = 2)]
  Name(String),
}

#[proto_message]
#[proto(skip_checks(all))]
#[proto(validate = |v| v.cel(prog_2()))]
struct MsgAndOneofDuplicateRules {
  #[proto(oneof(tags(1, 2)))]
  pub oneof: Option<OneofWithRule>,
}

#[test]
fn msg_and_oneof_duplicate_rules() {
  let mut file = ProtoFile::new("abc", "abc");

  file.with_messages([MsgAndOneofDuplicateRules::proto_schema()]);

  let package = Package {
    name: "abc".into(),
    files: vec![file],
  };

  assert!(package.check_unique_cel_rules().is_err());
}

#[proto_message]
#[proto(skip_checks(all))]
struct FieldAndOneofDuplicateRules {
  #[proto(oneof(tags(1, 2)))]
  pub oneof: Option<OneofWithRule>,
  #[proto(validate = |v| v.cel(prog_2()))]
  pub id: i32,
}

#[test]
fn field_and_oneof_duplicate_rules() {
  let mut file = ProtoFile::new("abc", "abc");

  file.with_messages([FieldAndOneofDuplicateRules::proto_schema()]);

  let package = Package {
    name: "abc".into(),
    files: vec![file],
  };

  assert!(package.check_unique_cel_rules().is_err());
}

#[proto_oneof]
#[proto(skip_checks(all))]
enum DuplicateRuleOneof {
  #[proto(tag = 1, validate = |v| v.cel(prog_1()))]
  Id(i32),
  #[proto(tag = 2, validate = |v| v.cel(prog_2()))]
  Name(String),
}

#[proto_message]
#[proto(skip_checks(all))]
struct OneofDuplicateRules {
  #[proto(oneof(tags(1, 2)))]
  pub oneof: Option<DuplicateRuleOneof>,
}

#[test]
fn oneof_duplicate_rules() {
  let mut file = ProtoFile::new("abc", "abc");

  file.with_messages([OneofDuplicateRules::proto_schema()]);

  let package = Package {
    name: "abc".into(),
    files: vec![file],
  };

  assert!(package.check_unique_cel_rules().is_err());
}

// This one should be okay because it's the same rule used twice, not
// two different rules with the same ID
#[proto_message]
#[proto(skip_checks(all))]
#[proto(validate = |v| v.cel(prog_1()))]
struct BenignDuplicateRules {
  #[proto(tag = 1, validate = |v| v.cel(prog_1()))]
  pub id: i32,
}

#[test]
fn benign_duplicate_rules() {
  let mut file = ProtoFile::new("abc", "abc");

  file.with_messages([BenignDuplicateRules::proto_schema()]);

  let package = Package {
    name: "abc".into(),
    files: vec![file],
  };

  assert!(package.check_unique_cel_rules().is_ok());
}
