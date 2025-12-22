use super::*;

// TODO: Check other cases for deep duplication search like in oneofs and top level messages mixed with fields

#[proto_message(direct)]
struct DuplicateRules {
  #[proto(tag = 1, validate = |v| v.cel(inline_cel_program!(id = "abc", msg = "hi", expr = "this == 0")).cel(inline_cel_program!(id = "abc", msg = "not hi", expr = "this == 0")))]
  pub id: i32,
}

#[test]
#[should_panic]
fn unique_rules() {
  let mut package = Package::new("abc");

  let mut file = ProtoFile::new("abc", "abc");

  file.add_messages([DuplicateRules::proto_schema()]);

  package.add_files([file]);

  package.check_unique_cel_rules();
}

// TODO: Check on fields and oneofs and top level

#[proto_message(direct, no_auto_test)]
struct BadRules {
  #[proto(tag = 1, validate = |v| v.cel(inline_cel_program!(id = "abc", msg = "hi", expr = "hi")))]
  pub id: i32,
}

#[test]
#[should_panic]
fn bad_rules() {
  BadRules::check_cel_programs();
}
