use super::*;

use_proto_file!(TESTING);

#[proto_oneof(no_auto_test)]
pub enum TestOneof {
  #[proto(tag = 100)]
  A(String),
  #[proto(tag = 200)]
  B(i32),
}

#[proto_message(no_auto_test)]
pub struct WrongTagsTest {
  #[proto(oneof(tags(1, 2)))]
  pub oneof: Option<TestOneof>,
}

#[test]
fn wrong_oneof_tags_check() {
  let err = WrongTagsTest::check_validators_consistency().unwrap_err();

  assert!(matches!(
    err.field_errors[0].errors[0],
    ConsistencyError::WrongOneofTags(_)
  ));
}

#[proto_message(no_auto_test)]
pub struct CorrectTagsTest {
  #[proto(oneof(tags(100, 200)))]
  pub oneof: Option<TestOneof>,
}

#[test]
fn correct_oneof_tags_check() {
  assert!(CorrectTagsTest::check_validators_consistency().is_ok());
}
