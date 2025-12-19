use super::*;

#[proto_message(direct)]
#[proto(package = "", file = "")]
struct MinPairs {
  #[proto(map(int32, int32), tag = 1, validate = |v| v.min_pairs(1))]
  pub items: HashMap<i32, i32>,
}

#[test]
fn min_pairs() {
  let mut msg = MinPairs {
    items: HashMap::default(),
  };

  let err = msg.validate().unwrap_err();

  assert_eq!(err.first().unwrap().rule_id(), "map.min_pairs");
}

#[proto_message(direct)]
#[proto(package = "", file = "")]
struct MaxPairs {
  #[proto(map(int32, int32), tag = 1, validate = |v| v.max_pairs(1))]
  pub items: HashMap<i32, i32>,
}

#[test]
fn max_pairs() {
  let mut map = HashMap::new();
  map.insert(1, 1);
  map.insert(2, 2);

  let mut msg = MaxPairs { items: map };

  let err = msg.validate().unwrap_err();

  assert_eq!(err.first().unwrap().rule_id(), "map.max_pairs");
}
