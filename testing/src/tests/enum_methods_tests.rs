use super::*;

#[proto_enum]
enum EnumMethodsTest {
  Unspecified,
  A,
  B,
}

#[test]
fn name() {
  assert_eq_pretty!(EnumMethodsTest::proto_name(), "EnumMethodsTest");

  let unspecified = EnumMethodsTest::default();

  assert!(unspecified.is_unspecified());

  let variant = EnumMethodsTest::A;

  assert!(!variant.is_unspecified());

  assert!(EnumMethodsTest::is_known_variant(1));

  let fallback = EnumMethodsTest::from_int_or_default(15);

  assert_eq_pretty!(fallback, unspecified);

  assert_eq_pretty!(unspecified.as_proto_name(), "ENUM_METHODS_TEST_UNSPECIFIED");
  assert_eq_pretty!(
    EnumMethodsTest::from_proto_name("ENUM_METHODS_TEST_UNSPECIFIED").unwrap(),
    unspecified
  );
}
