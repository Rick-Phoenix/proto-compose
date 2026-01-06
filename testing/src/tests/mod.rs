mod cel_tests;
mod oneof_tags_tests;
mod rendering_tests;

use bytes::Bytes;
use prelude::{test_utils::*, *};
use similar_asserts::assert_eq as assert_eq_pretty;

proto_package!(TESTING_PKG, name = "testing", no_cel_test);

define_proto_file!(TESTING, file = "testing", package = TESTING_PKG);
