use proto_types::{Duration, Timestamp};

use crate::proto::TimestampRules;

use super::*;

fn one_sec() -> Timestamp {
  Timestamp {
    seconds: 1,
    nanos: 0,
  }
}

fn minus_one_sec() -> Timestamp {
  Timestamp {
    seconds: -1,
    nanos: 0,
  }
}

fn zero_sec() -> Timestamp {
  Timestamp::default()
}

fn future() -> Timestamp {
  Timestamp::now()
    + Duration {
      seconds: 300,
      nanos: 0,
    }
}

#[allow(unused)]
#[test]
fn timestamp_tests() {
  let mut msg = TimestampRules {
    const_test: Some(zero_sec()),
    lt_test: Some(minus_one_sec()),
    lte_test: Some(zero_sec()),
    gt_test: Some(one_sec()),
    gte_test: Some(zero_sec()),
    required_test: Some(zero_sec()),
    ignore_always_test: Some(one_sec()),
    within_test: Some(Timestamp::now()),
    lt_now_test: Some(zero_sec()),
    gt_now_test: Some(future()),
  };

  let baseline = msg;

  assert!(msg.validate().is_ok(), "basic validation");

  macro_rules! assert_violation {
    ($violation:expr, $error:literal) => {
      assert_violation_id(&msg, $violation, $error);
      msg = baseline;
    };
  }

  msg.const_test = Some(one_sec());
  assert_violation!("timestamp.const", "const rule");

  msg.lt_test = Some(zero_sec());
  assert_violation!("timestamp.lt", "lt rule");

  msg.lte_test = Some(one_sec());
  assert_violation!("timestamp.lte", "lte rule");

  msg.gt_test = Some(zero_sec());
  assert_violation!("timestamp.gt", "gt rule");

  msg.gte_test = Some(minus_one_sec());
  assert_violation!("timestamp.gte", "gte rule");

  msg.within_test = Some(minus_one_sec());
  assert_violation!("timestamp.within", "within rule");

  msg.lt_now_test = Some(future());
  assert_violation!("timestamp.lt_now", "lt_now rule");

  msg.gt_now_test = Some(minus_one_sec());
  assert_violation!("timestamp.gt_now", "gt_now rule");

  msg.required_test = None;
  assert_violation!("required", "required rule");
}
