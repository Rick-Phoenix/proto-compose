use super::*;

#[proto_oneof(direct)]
pub enum TestOneof {
  #[proto(tag = 100)]
  A(String),
  #[proto(tag = 200)]
  B(i32),
}

#[proto_message(direct, no_auto_test)]
pub struct ShouldPanicTest {
  #[proto(oneof(tags(1, 2)))]
  pub oneof: Option<TestOneof>,
}

#[test]
#[should_panic]
fn should_panic_oneof_tags_check() {
  ShouldPanicTest::check_oneofs_tags();
}

#[proto_message(direct, no_auto_test)]
pub struct ShouldNotPanicTest {
  #[proto(oneof(tags(100, 200)))]
  pub oneof: Option<TestOneof>,
}

#[test]
fn should_not_panic_oneof_tags_check() {
  ShouldNotPanicTest::check_oneofs_tags();
}
