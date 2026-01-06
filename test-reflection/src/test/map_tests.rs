use maplit::hashmap;

use super::*;

#[test]
fn map_tests() {
  let mut msg = MapTests {
    min_pairs_test: hashmap! { 1 => 1 },
    max_pairs_test: hashmap! { 1 => 1 },
    keys_test: hashmap! { 15 => 1 },
    values_test: hashmap! { 1 => 15 },
    cel_test: hashmap! { 1 => 1 },
  };
  let baseline = msg.clone();

  assert!(msg.validate().is_ok(), "basic validation");

  macro_rules! assert_violation {
    ($violation:expr, $error:expr) => {
      assert_violation_id(&msg, $violation, $error);
      msg = baseline.clone();
    };
  }

  macro_rules! assert_violation_path {
    ($violation:expr, $error:literal) => {
      assert_eq!(full_rule_id_path(&msg), $violation, $error);
      msg = baseline.clone();
    };
  }

  msg.min_pairs_test = hashmap! {};
  assert_violation!("map.min_pairs", "min pairs rule");

  msg.max_pairs_test = hashmap! { 1 => 1, 2 => 2 };
  assert_violation!("map.max_pairs", "max pairs rule");

  msg.keys_test = hashmap! { 1 => 1 };
  assert_violation_path!("map.keys.int32.gte", "keys rule");

  msg.values_test = hashmap! { 1 => 1 };
  assert_violation_path!("map.values.int32.gte", "values rule");

  msg.keys_test = hashmap! { 16 => 1 };
  assert_violation_path!("map.keys.cel", "keys cel rule");

  msg.values_test = hashmap! { 1 => 16 };
  assert_violation_path!("map.values.cel", "values cel rule");

  msg.cel_test = hashmap! { 10 => 1 };
  assert_violation!("cel_rule", "cel rule");

  msg.cel_test = hashmap! {};
  assert!(msg.validate().is_ok(), "Should ignore empty map");
}
