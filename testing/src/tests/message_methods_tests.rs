use super::*;

#[proto_message]
#[proto(skip_checks(all))]
struct TestMsg {
  id: i32,
}

#[proto_message]
#[proto(skip_checks(all))]
#[proto(parent_message = TestMsg)]
struct NestedMsg {
  id: i32,
}

#[test]
fn message_methods() {
  // For top level messages, the name is the same as the short name
  assert_eq_pretty!(TestMsg::proto_name(), TestMsg::SHORT_NAME);

  // For nested messages, it shouldn't be
  assert_ne!(NestedMsg::proto_name(), NestedMsg::SHORT_NAME);

  assert_eq_pretty!("TestMsg.NestedMsg", NestedMsg::proto_name());

  assert_eq_pretty!(
    format!("{}.{}", TestMsg::PACKAGE, TestMsg::proto_name()),
    TestMsg::full_name()
  );
  assert_eq_pretty!(
    format!("{}.{}", NestedMsg::PACKAGE, NestedMsg::proto_name()),
    NestedMsg::full_name()
  );

  assert_eq_pretty!(
    format!("/{}.{}", TestMsg::PACKAGE, TestMsg::proto_name()),
    TestMsg::type_url()
  );
  assert_eq_pretty!(
    format!("/{}.{}", NestedMsg::PACKAGE, NestedMsg::proto_name()),
    NestedMsg::type_url()
  );
}
