#![allow(
  clippy::needless_pass_by_value,
  clippy::struct_field_names,
  clippy::use_self,
  clippy::derive_partial_eq_without_eq
)]

mod cel_tests;
mod oneof_tags_tests;
mod validation_tests;

mod schema_tests;
mod tolerances_tests;

use ::bytes::Bytes;
use prelude::{test_utils::*, *};
use similar_asserts::assert_eq as assert_eq_pretty;
use std::collections::HashMap;

proto_package!(TESTING_PKG, name = "testing", no_cel_test);

define_proto_file!(TESTING, name = "testing.proto", package = TESTING_PKG);

#[track_caller]
pub(crate) fn assert_violation_id(msg: &impl ValidatedMessage, expected: &str, error: &str) {
  let violations = msg.validate().unwrap_err();

  assert_eq!(violations.len(), 1, "Expected a single violation");
  assert_eq!(violations.first().unwrap().rule_id(), expected, "{error}");
}

#[proto_message(no_auto_test)]
pub struct DirectMsg {
  pub id: i32,
}

#[proto_message(proxied, no_auto_test)]
pub struct ProxiedMsg {
  pub id: i32,
}
