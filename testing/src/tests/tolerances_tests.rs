#![allow(unused_assignments)]

use proto_types::{Duration, Timestamp};

use super::*;

#[proto_message(no_auto_test)]
pub struct TolerancesTests {
  #[proto(validate = |v| v.const_(12.0).abs_tolerance(0.0001))]
  pub float_tolerance: f64,
  #[proto(timestamp, validate = |v| v.gt_now().now_tolerance(Duration { seconds: 5, nanos: 0 }))]
  pub timestamp_tolerance: Option<Timestamp>,
}

#[test]
fn tolerances_tests() {
  let mut msg = TolerancesTests {
    float_tolerance: 12.00001,
    timestamp_tolerance: Some(Timestamp::now() - Duration::new(3, 0)),
  };
  let baseline = msg.clone();

  assert!(msg.validate().is_ok(), "basic validation");

  macro_rules! assert_violation {
    ($violation:expr, $error:literal) => {
      assert_violation_id(&msg, $violation, $error);
      msg = baseline.clone();
    };
  }

  msg.float_tolerance = 12.00011;
  assert_violation!("double.const", "float tolerance");

  msg.timestamp_tolerance = Some(Timestamp::now() - Duration::new(6, 0));
  assert_violation!("timestamp.gt_now", "timestamp gt_now tolerance");
}
