use super::*;

use_proto_file!(TESTING);

#[proto_oneof]
#[proto(skip_checks(all))]
pub enum TestOneof {
  #[proto(tag = 100)]
  A(String),
  #[proto(tag = 200)]
  B(i32),
}

#[proto_message]
#[proto(skip_checks(all))]
pub struct WrongTagsTest {
  #[proto(oneof(tags(1, 2)))]
  pub oneof: Option<TestOneof>,
}

#[test]
fn wrong_oneof_tags_check() {
  assert!(WrongTagsTest::check_oneofs_tags().is_err())
}

#[proto_message]
#[proto(skip_checks(all))]
pub struct CorrectTagsTest {
  #[proto(oneof(tags(100, 200)))]
  pub oneof: Option<TestOneof>,
}

#[test]
fn correct_oneof_tags_check() {
  assert!(CorrectTagsTest::check_oneofs_tags().is_ok());
}
