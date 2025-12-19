use super::*;

#[proto_message(direct)]
#[proto(package = "", file = "")]
struct DuplicateRules {
  #[proto(tag = 1, validate = |v| v.cel(inline_cel_program!(id = "abc", msg = "hi", expr = "hi")).cel(inline_cel_program!(id = "abc", msg = "not hi", expr = "not hi")))]
  pub id: i32,
}

#[test]
fn unique_rules() {
  let mut package = Package::new("abc");

  let mut file = ProtoFile::new("abc", "abc");

  file.add_messages([DuplicateRules::proto_schema()]);

  package.add_files([file]);

  assert!(package.check_unique_cel_rules().is_err());
}
