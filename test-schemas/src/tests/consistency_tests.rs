use super::*;

#[test]
fn bad_field_rules() {
  let MessageTestError {
    message_full_name,
    field_errors,
    top_level_errors,
  } = BadFieldRules::check_validators_consistency().unwrap_err();

  assert_eq_pretty!(message_full_name, "BadFieldRules");
  assert_eq_pretty!(field_errors.len(), 1);
  assert_eq_pretty!(top_level_errors.len(), 0);
}

#[test]
fn bad_msg_rules() {
  let MessageTestError {
    message_full_name,
    field_errors,
    top_level_errors,
  } = BadMsgRules::check_validators_consistency().unwrap_err();

  assert_eq_pretty!(message_full_name, "BadMsgRules");
  assert_eq_pretty!(field_errors.len(), 0);
  assert_eq_pretty!(top_level_errors.len(), 1);
}

#[test]
fn bad_oneof_rules() {
  let OneofErrors {
    oneof_name,
    field_errors: errors,
  } = BadCelOneof::check_validators_consistency().unwrap_err();

  assert_eq_pretty!(oneof_name, "BadCelOneof");
  assert_eq_pretty!(errors.len(), 1);
  assert!(matches!(errors[0].errors[0], ConsistencyError::CelError(_)));
}

#[proto_message(no_auto_test)]
pub struct LtGtError {
  #[proto(validate = |v| v.gt(2).lt(2))]
  gt_lt: i32,
  #[proto(validate = |v| v.gte(3).lt(2))]
  gte_lt: i32,
  #[proto(validate = |v| v.gt(2).lte(1))]
  gt_lte: i32,
  #[proto(validate = |v| v.gte(2).lte(1))]
  gte_lte: i32,
  #[proto(validate = |v| v.gte(2).gt(2))]
  gt_gte: i32,
  #[proto(validate = |v| v.lte(2).lt(2))]
  lt_lte: i32,
}

#[test]
fn lt_gt_error() {
  let MessageTestError { field_errors, .. } =
    LtGtError::check_validators_consistency().unwrap_err();

  assert_eq_pretty!(
    field_errors[0].errors[0],
    ConsistencyError::ContradictoryInput("Lt cannot be smaller than or equal to Gt".to_string())
  );
  assert_eq_pretty!(
    field_errors[1].errors[0],
    ConsistencyError::ContradictoryInput("Lt cannot be smaller than or equal to Gte".to_string())
  );
  assert_eq_pretty!(
    field_errors[2].errors[0],
    ConsistencyError::ContradictoryInput("Lte cannot be smaller than or equal to Gt".to_string())
  );
  assert_eq_pretty!(
    field_errors[3].errors[0],
    ConsistencyError::ContradictoryInput("Lte cannot be smaller than Gte".to_string())
  );
  assert_eq_pretty!(
    field_errors[4].errors[0],
    ConsistencyError::ContradictoryInput("Gt and Gte cannot be used together.".to_string())
  );
  assert_eq_pretty!(
    field_errors[5].errors[0],
    ConsistencyError::ContradictoryInput("Lt and Lte cannot be used together.".to_string())
  );
}

#[proto_message(no_auto_test)]
pub struct FloatLtGtError {
  #[proto(validate = |v| v.gt(2.0).lt(2.0))]
  gt_lt: f32,
  #[proto(validate = |v| v.gte(3.0).lt(2.0))]
  gte_lt: f32,
  #[proto(validate = |v| v.gt(2.0).lte(1.0))]
  gt_lte: f32,
  #[proto(validate = |v| v.gte(2.0).lte(1.0))]
  gte_lte: f32,
  #[proto(validate = |v| v.gte(2.0).gt(2.0))]
  gt_gte: f32,
  #[proto(validate = |v| v.lte(2.0).lt(2.0))]
  lt_lte: f32,
}

#[test]
fn float_lt_gt_error() {
  let MessageTestError { field_errors, .. } =
    FloatLtGtError::check_validators_consistency().unwrap_err();

  assert_eq_pretty!(
    field_errors[0].errors[0],
    ConsistencyError::ContradictoryInput("Lt cannot be smaller than or equal to Gt".to_string())
  );
  assert_eq_pretty!(
    field_errors[1].errors[0],
    ConsistencyError::ContradictoryInput("Lt cannot be smaller than or equal to Gte".to_string())
  );
  assert_eq_pretty!(
    field_errors[2].errors[0],
    ConsistencyError::ContradictoryInput("Lte cannot be smaller than or equal to Gt".to_string())
  );
  assert_eq_pretty!(
    field_errors[3].errors[0],
    ConsistencyError::ContradictoryInput("Lte cannot be smaller than Gte".to_string())
  );
  assert_eq_pretty!(
    field_errors[4].errors[0],
    ConsistencyError::ContradictoryInput("Gt and Gte cannot be used together.".to_string())
  );
  assert_eq_pretty!(
    field_errors[5].errors[0],
    ConsistencyError::ContradictoryInput("Lt and Lte cannot be used together.".to_string())
  );
}

fn two_secs() -> Duration {
  Duration {
    seconds: 2,
    nanos: 0,
  }
}

fn one_sec() -> Duration {
  Duration {
    seconds: 1,
    nanos: 0,
  }
}

#[proto_message(no_auto_test)]
pub struct DurationLtGtError {
  #[proto(duration, validate = |v| v.gt(two_secs()).lt(two_secs()))]
  gt_lt: Option<Duration>,
  #[proto(duration, validate = |v| v.gte(two_secs()).lt(two_secs()))]
  gte_lt: Option<Duration>,
  #[proto(duration, validate = |v| v.gt(two_secs()).lte(one_sec()))]
  gt_lte: Option<Duration>,
  #[proto(duration, validate = |v| v.gte(two_secs()).lte(one_sec()))]
  gte_lte: Option<Duration>,
  #[proto(duration, validate = |v| v.gte(two_secs()).gt(two_secs()))]
  gt_gte: Option<Duration>,
  #[proto(duration, validate = |v| v.lte(two_secs()).lt(two_secs()))]
  lt_lte: Option<Duration>,
}

#[test]
fn duration_lt_gt_error() {
  let MessageTestError { field_errors, .. } =
    DurationLtGtError::check_validators_consistency().unwrap_err();

  assert_eq_pretty!(
    field_errors[0].errors[0],
    ConsistencyError::ContradictoryInput("Lt cannot be smaller than or equal to Gt".to_string())
  );
  assert_eq_pretty!(
    field_errors[1].errors[0],
    ConsistencyError::ContradictoryInput("Lt cannot be smaller than or equal to Gte".to_string())
  );
  assert_eq_pretty!(
    field_errors[2].errors[0],
    ConsistencyError::ContradictoryInput("Lte cannot be smaller than or equal to Gt".to_string())
  );
  assert_eq_pretty!(
    field_errors[3].errors[0],
    ConsistencyError::ContradictoryInput("Lte cannot be smaller than Gte".to_string())
  );
  assert_eq_pretty!(
    field_errors[4].errors[0],
    ConsistencyError::ContradictoryInput("Gt and Gte cannot be used together.".to_string())
  );
  assert_eq_pretty!(
    field_errors[5].errors[0],
    ConsistencyError::ContradictoryInput("Lt and Lte cannot be used together.".to_string())
  );
}

fn now() -> Timestamp {
  Timestamp::now()
}

fn past() -> Timestamp {
  Default::default()
}

#[proto_message(no_auto_test)]
pub struct TimestampLtGtError {
  #[proto(timestamp, validate = |v| v.gt(now()).lt(past()))]
  gt_lt: Option<Timestamp>,
  #[proto(timestamp, validate = |v| v.gte(now()).lt(past()))]
  gte_lt: Option<Timestamp>,
  #[proto(timestamp, validate = |v| v.gt(now()).lte(past()))]
  gt_lte: Option<Timestamp>,
  #[proto(timestamp, validate = |v| v.gte(now()).lte(past()))]
  gte_lte: Option<Timestamp>,
  #[proto(timestamp, validate = |v| v.gte(now()).gt(now()))]
  gt_gte: Option<Timestamp>,
  #[proto(timestamp, validate = |v| v.lte(now()).lt(now()))]
  lt_lte: Option<Timestamp>,
  #[proto(timestamp, validate = |v| v.lt(now()).lt_now())]
  lt_ltnow: Option<Timestamp>,
  #[proto(timestamp, validate = |v| v.lte(now()).lt_now())]
  lte_ltnow: Option<Timestamp>,
  #[proto(timestamp, validate = |v| v.gt(now()).gt_now())]
  gt_gtnow: Option<Timestamp>,
  #[proto(timestamp, validate = |v| v.gte(now()).gt_now())]
  gte_gtnow: Option<Timestamp>,
  #[proto(timestamp, validate = |v| v.gt_now().lt_now())]
  gtnow_ltnow: Option<Timestamp>,
}

#[test]
fn timestamp_lt_gt_error() {
  let MessageTestError { field_errors, .. } =
    TimestampLtGtError::check_validators_consistency().unwrap_err();

  assert_eq_pretty!(
    field_errors[0].errors[0],
    ConsistencyError::ContradictoryInput("Lt cannot be smaller than or equal to Gt".to_string())
  );
  assert_eq_pretty!(
    field_errors[1].errors[0],
    ConsistencyError::ContradictoryInput("Lt cannot be smaller than or equal to Gte".to_string())
  );
  assert_eq_pretty!(
    field_errors[2].errors[0],
    ConsistencyError::ContradictoryInput("Lte cannot be smaller than or equal to Gt".to_string())
  );
  assert_eq_pretty!(
    field_errors[3].errors[0],
    ConsistencyError::ContradictoryInput("Lte cannot be smaller than Gte".to_string())
  );
  assert_eq_pretty!(
    field_errors[4].errors[0],
    ConsistencyError::ContradictoryInput("Gt and Gte cannot be used together.".to_string())
  );
  assert_eq_pretty!(
    field_errors[5].errors[0],
    ConsistencyError::ContradictoryInput("Lt and Lte cannot be used together.".to_string())
  );
  assert_eq_pretty!(
    field_errors[6].errors[0],
    ConsistencyError::ContradictoryInput("`lt_now` cannot be used with `lt` or `lte`".to_string())
  );
  assert_eq_pretty!(
    field_errors[7].errors[0],
    ConsistencyError::ContradictoryInput("`lt_now` cannot be used with `lt` or `lte`".to_string())
  );
  assert_eq_pretty!(
    field_errors[8].errors[0],
    ConsistencyError::ContradictoryInput("`gt_now` cannot be used with `gt` or `gte`".to_string())
  );
  assert_eq_pretty!(
    field_errors[9].errors[0],
    ConsistencyError::ContradictoryInput("`gt_now` cannot be used with `gt` or `gte`".to_string())
  );
  assert_eq_pretty!(
    field_errors[10].errors[0],
    ConsistencyError::ContradictoryInput(
      "`lt_now` and `gt_now` cannot be used together".to_string()
    )
  );
}

#[proto_message(no_auto_test)]
pub struct ListErrors {
  #[proto(validate = |v| v.in_([1]).not_in([1]))]
  int: i32,
  #[proto(validate = |v| v.in_([1.0]).not_in([1.0]))]
  float: f32,
  #[proto(duration, validate = |v| v.in_([one_sec()]).not_in([one_sec()]))]
  duration: Option<Duration>,
  #[proto(any, validate = |v| v.in_(["abc"]).not_in(["abc"]))]
  any: Option<Any>,
  #[proto(string, validate = |v| v.in_(["abc"]).not_in(["abc"]))]
  string: String,
  #[proto(bytes, validate = |v| v.in_([b"abc"]).not_in([b"abc"]))]
  bytes: Bytes,
  #[proto(enum_(TestEnum), validate = |v| v.in_([1]).not_in([1]))]
  enum_: i32,
  #[proto(enum_(TestEnum), validate = |v| v.in_([100]))]
  enum_with_unknown_tag: i32,
  #[proto(field_mask, validate = |v| v.in_(["abc"]).not_in(["abc"]))]
  field_mask: Option<FieldMask>,
}

#[test]
fn list_errors() {
  let MessageTestError { field_errors, .. } =
    ListErrors::check_validators_consistency().unwrap_err();

  for (i, err) in field_errors.iter().enumerate() {
    if i == 7 {
      assert_eq_pretty!(
        err.errors[0],
        ConsistencyError::ContradictoryInput(
          "Number 100 is in the allowed list but it does not belong to the enum TestEnum"
            .to_string()
        )
      );
    } else {
      assert!(matches!(
        err.errors[0],
        ConsistencyError::OverlappingLists(_)
      ));
    }
  }
}

#[proto_message(no_auto_test)]
pub struct LengthRulesErrors {
  #[proto(validate = |v| v.min_items(3).max_items(1))]
  repeated: Vec<i32>,
  #[proto(map(int32, int32), validate = |v| v.min_pairs(2).max_pairs(1))]
  map: HashMap<i32, i32>,
  #[proto(bytes, validate = |v| v.min_len(2).max_len(1))]
  bytes_min_max_len: Bytes,
  #[proto(bytes, validate = |v| v.min_len(2).len(1))]
  bytes_len_min_len: Bytes,
  #[proto(bytes, validate = |v| v.max_len(2).len(1))]
  bytes_len_max_len: Bytes,
  #[proto(string, validate = |v| v.min_len(2).max_len(1))]
  string_min_max_len: String,
  #[proto(string, validate = |v| v.min_len(2).len(1))]
  string_len_min_len: String,
  #[proto(string, validate = |v| v.max_len(2).len(1))]
  string_len_max_len: String,
  #[proto(string, validate = |v| v.min_bytes(2).max_bytes(1))]
  string_min_max_len_bytes: String,
  #[proto(string, validate = |v| v.min_bytes(2).len_bytes(1))]
  string_len_bytes_min_bytes: String,
  #[proto(string, validate = |v| v.max_bytes(2).len_bytes(1))]
  string_len_bytes_max_bytes: String,
}

#[test]
fn length_rules_errors() {
  let MessageTestError { field_errors, .. } =
    LengthRulesErrors::check_validators_consistency().unwrap_err();

  assert_eq_pretty!(
    field_errors[0].errors[0],
    ConsistencyError::ContradictoryInput("min_items cannot be greater than max_items".to_string())
  );
  assert_eq_pretty!(
    field_errors[1].errors[0],
    ConsistencyError::ContradictoryInput("min_pairs cannot be greater than max_pairs".to_string())
  );
  assert_eq_pretty!(
    field_errors[2].errors[0],
    ConsistencyError::ContradictoryInput("min_len cannot be greater than max_len".to_string())
  );
  assert_eq_pretty!(
    field_errors[3].errors[0],
    ConsistencyError::ContradictoryInput("len cannot be used with min_len".to_string())
  );
  assert_eq_pretty!(
    field_errors[4].errors[0],
    ConsistencyError::ContradictoryInput("len cannot be used with max_len".to_string())
  );
  assert_eq_pretty!(
    field_errors[5].errors[0],
    ConsistencyError::ContradictoryInput("min_len cannot be greater than max_len".to_string())
  );
  assert_eq_pretty!(
    field_errors[6].errors[0],
    ConsistencyError::ContradictoryInput("len cannot be used with min_len".to_string())
  );
  assert_eq_pretty!(
    field_errors[7].errors[0],
    ConsistencyError::ContradictoryInput("len cannot be used with max_len".to_string())
  );
  assert_eq_pretty!(
    field_errors[8].errors[0],
    ConsistencyError::ContradictoryInput("min_bytes cannot be greater than max_bytes".to_string())
  );
  assert_eq_pretty!(
    field_errors[9].errors[0],
    ConsistencyError::ContradictoryInput("len_bytes cannot be used with min_bytes".to_string())
  );
  assert_eq_pretty!(
    field_errors[10].errors[0],
    ConsistencyError::ContradictoryInput("len_bytes cannot be used with max_bytes".to_string())
  );
}

#[proto_message(no_auto_test)]
pub struct StringErrors {
  #[proto(validate = |v| v.contains("abc").not_contains("abc"))]
  contains_not_contains: String,
}

#[test]
fn string_errors() {
  let MessageTestError { field_errors, .. } =
    StringErrors::check_validators_consistency().unwrap_err();

  assert_eq_pretty!(
    field_errors[0].errors[0],
    ConsistencyError::ContradictoryInput("`not_contains` is a substring of `contains`".to_string())
  );
}

#[proto_message(no_auto_test)]
pub struct ConstWithOtherRules {
  #[proto(validate = |v| v.const_("abc").min_len(3))]
  string: String,
  #[proto(bytes, validate = |v| v.const_(b"abc").min_len(3))]
  bytes: Bytes,
  #[proto(validate = |v| v.const_(3).gt(2))]
  int: i32,
  #[proto(validate = |v| v.const_(3.0).gt(2.0))]
  float: f32,
  #[proto(duration, validate = |v| v.const_(Duration::new(1, 0)).gt(Duration::default()))]
  duration: Option<Duration>,
  #[proto(timestamp, validate = |v| v.const_(Timestamp::new(1, 0)).gt(Timestamp::default()))]
  timestamp: Option<Timestamp>,
  #[proto(field_mask, validate = |v| v.const_(["abc"]).in_(["abc"]))]
  field_mask: Option<FieldMask>,
  #[proto(enum_(TestEnum), validate = |v| v.const_(1).defined_only())]
  enum_field: i32,
}

#[test]
fn const_with_other_rules() {
  let MessageTestError { field_errors, .. } =
    ConstWithOtherRules::check_validators_consistency().unwrap_err();

  for err in field_errors {
    assert_eq_pretty!(err.errors[0], ConsistencyError::ConstWithOtherRules);
  }
}
