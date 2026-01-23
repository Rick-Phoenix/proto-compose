use super::*;

#[allow(clippy::use_self)]
#[proto_message(no_auto_test)]
// Using a CEL rule implicitly tests that recursive CEL conversions are working fine
#[proto(validate = |v| v.cel(cel_program!(id = "id_is_1", msg = "abc", expr = "this.id == 1")))]
struct RecursiveMsg {
  id: i32,
  #[proto(message)]
  recursive: Option<Box<RecursiveMsg>>,
  #[proto(message)]
  recursive_2nd_degree: Option<RecursiveMsg2>,
  #[proto(oneof(tags(1, 2)))]
  recursive_oneof: Option<RecursiveOneof>,
}

#[proto_message(no_auto_test)]
struct RecursiveMsg2 {
  #[proto(message)]
  recursive: Option<Box<RecursiveMsg>>,
}

#[proto_oneof(no_auto_test)]
enum RecursiveOneof {
  #[proto(tag = 1, message)]
  Recursive1Deg(Box<RecursiveMsg>),
  #[proto(tag = 2, message)]
  Recursive2Deg(RecursiveMsg2),
}

#[allow(unused, clippy::redundant_clone)]
#[test]
fn recursion_tests() {
  let base = RecursiveMsg {
    id: 1,
    recursive: None,
    recursive_2nd_degree: None,
    recursive_oneof: None,
  };
  let base_2nd_deg = RecursiveMsg2 {
    recursive: Some(base.clone().into()),
  };

  let invalid = RecursiveMsg::default();
  let invalid_2nd_deg = RecursiveMsg2 {
    recursive: Some(invalid.clone().into()),
  };

  let mut msg = RecursiveMsg {
    id: 1,
    recursive: Some(base.clone().into()),
    recursive_2nd_degree: Some(base_2nd_deg.clone()),
    recursive_oneof: Some(RecursiveOneof::Recursive1Deg(base.clone().into())),
  };
  let baseline = msg.clone();

  assert!(msg.validate().is_ok(), "basic validation with recursion");

  msg.recursive_oneof = Some(RecursiveOneof::Recursive2Deg(base_2nd_deg.clone()));

  assert!(
    msg.validate().is_ok(),
    "basic validation with 2nd deg recursion"
  );

  msg.recursive = Some(invalid.clone().into());
  assert_violation_id(&msg, "id_is_1", "1 deg recursion validation");
  msg = baseline.clone();

  msg.recursive_2nd_degree = Some(invalid_2nd_deg.clone());
  assert_violation_id(&msg, "id_is_1", "2 deg recursion validation");
  msg = baseline.clone();

  msg.recursive_oneof = Some(RecursiveOneof::Recursive1Deg(invalid.clone().into()));
  assert_violation_id(&msg, "id_is_1", "1 deg oneof recursion validation");
  msg = baseline.clone();

  msg.recursive_oneof = Some(RecursiveOneof::Recursive2Deg(invalid_2nd_deg.clone()));
  assert_violation_id(&msg, "id_is_1", "2 deg oneof recursion validation");
  msg = baseline.clone();
}

#[allow(clippy::use_self)]
#[proto_message(proxied, no_auto_test)]
// Using a CEL rule implicitly tests that recursive CEL conversions are working fine
#[proto(validate = |v| v.cel(cel_program!(id = "id_is_1", msg = "abc", expr = "this.id == 1")))]
pub struct RecursiveProxiedMsg {
  id: i32,
  #[proto(message(proxied))]
  recursive: Option<Box<RecursiveProxiedMsg>>,
  #[proto(message(proxied))]
  recursive_2nd_degree: Option<RecursiveProxiedMsg2>,
  #[proto(oneof(proxied, tags(1, 2)))]
  recursive_oneof: Option<RecursiveProxiedOneof>,
}

#[proto_message(proxied, no_auto_test)]
pub struct RecursiveProxiedMsg2 {
  #[proto(message(proxied))]
  recursive: Option<Box<RecursiveProxiedMsg>>,
}

#[proto_oneof(proxied, no_auto_test)]
pub enum RecursiveProxiedOneof {
  #[proto(tag = 1, message(proxied))]
  Recursive1Deg(Box<RecursiveProxiedMsg>),
  #[proto(tag = 2, message(proxied))]
  Recursive2Deg(RecursiveProxiedMsg2),
}

#[allow(unused, clippy::redundant_clone)]
#[test]
fn proxied_recursion_tests() {
  let base = RecursiveProxiedMsgProto {
    id: 1,
    recursive: None,
    recursive_2nd_degree: None,
    recursive_oneof: None,
  };
  let base_2nd_deg = RecursiveProxiedMsg2Proto {
    recursive: Some(base.clone().into()),
  };

  let invalid = RecursiveProxiedMsgProto::default();
  let invalid_2nd_deg = RecursiveProxiedMsg2Proto {
    recursive: Some(invalid.clone().into()),
  };

  let mut msg = RecursiveProxiedMsgProto {
    id: 1,
    recursive: Some(base.clone().into()),
    recursive_2nd_degree: Some(base_2nd_deg.clone()),
    recursive_oneof: Some(RecursiveProxiedOneofProto::Recursive1Deg(
      base.clone().into(),
    )),
  };
  let baseline = msg.clone();

  assert!(msg.validate().is_ok(), "basic validation with recursion");

  msg.recursive_oneof = Some(RecursiveProxiedOneofProto::Recursive2Deg(
    base_2nd_deg.clone(),
  ));

  assert!(
    msg.validate().is_ok(),
    "basic validation with 2nd deg recursion"
  );

  msg.recursive = Some(invalid.clone().into());
  assert_violation_id(&msg, "id_is_1", "1 deg recursion validation");
  msg = baseline.clone();

  msg.recursive_2nd_degree = Some(invalid_2nd_deg.clone());
  assert_violation_id(&msg, "id_is_1", "2 deg recursion validation");
  msg = baseline.clone();

  msg.recursive_oneof = Some(RecursiveProxiedOneofProto::Recursive1Deg(
    invalid.clone().into(),
  ));
  assert_violation_id(&msg, "id_is_1", "1 deg oneof recursion validation");
  msg = baseline.clone();

  msg.recursive_oneof = Some(RecursiveProxiedOneofProto::Recursive2Deg(
    invalid_2nd_deg.clone(),
  ));
  assert_violation_id(&msg, "id_is_1", "2 deg oneof recursion validation");
  msg = baseline.clone();
}

#[test]
fn recursive_conversions() {
  let base = RecursiveProxiedMsgProto {
    id: 1,
    recursive: None,
    recursive_2nd_degree: None,
    recursive_oneof: None,
  };
  let base_2nd_deg = RecursiveProxiedMsg2Proto {
    recursive: Some(base.clone().into()),
  };

  let mut msg = RecursiveProxiedMsgProto {
    id: 1,
    recursive: Some(base.clone().into()),
    recursive_2nd_degree: Some(base_2nd_deg.clone()),
    recursive_oneof: Some(RecursiveProxiedOneofProto::Recursive1Deg(base.into())),
  };

  let proxy = msg.clone().into_proxy();
  assert_eq_pretty!(proxy.id, 1);
  assert_eq_pretty!(proxy.recursive.unwrap().id, 1);
  assert_eq_pretty!(
    proxy
      .recursive_2nd_degree
      .unwrap()
      .recursive
      .unwrap()
      .id,
    1
  );
  let Some(RecursiveProxiedOneof::Recursive1Deg(val)) = proxy.recursive_oneof else {
    panic!()
  };
  assert_eq_pretty!(val.id, 1);

  msg.recursive_oneof = Some(RecursiveProxiedOneofProto::Recursive2Deg(base_2nd_deg));

  let proxy2 = msg.into_proxy();
  assert_eq_pretty!(proxy2.id, 1);
  assert_eq_pretty!(proxy2.recursive.unwrap().id, 1);
  assert_eq_pretty!(
    proxy2
      .recursive_2nd_degree
      .unwrap()
      .recursive
      .unwrap()
      .id,
    1
  );
  let Some(RecursiveProxiedOneof::Recursive2Deg(val)) = proxy2.recursive_oneof else {
    panic!()
  };
  assert_eq_pretty!(val.recursive.unwrap().id, 1);
}
