#![allow(clippy::struct_field_names)]

use bytes::Bytes;
use prelude::{
  cel_program, define_proto_file, proto_message, proto_package,
  proto_types::{Any, Duration, FieldMask, Timestamp},
};
use prelude::{proto_enum, proto_oneof};
use std::collections::HashMap;

proto_package!(TEST_SCHEMAS, name = "test_schemas.v1", no_cel_test);
define_proto_file!(
  TEST_SCHEMAS_FILE,
  file = "test_schemas.proto",
  package = TEST_SCHEMAS
);

#[proto_message(no_auto_test)]
pub struct ParentMessage {
  #[proto(message)]
  pub nested_message: Option<NestedMessage>,
  #[proto(enum_(NestedEnum))]
  pub nested_enum: i32,
}

#[proto_enum]
#[proto(parent_message = NestedMessage)]
pub enum NestedEnum {
  A,
  B,
  C,
}

#[proto_message(no_auto_test)]
#[proto(parent_message = ParentMessage)]
pub struct NestedMessage {
  #[proto(enum_(NestedEnum))]
  pub nested_enum: i32,
}

#[proto_message(no_auto_test)]
pub struct TolerancesTests {
  #[proto(validate = |v| v.const_(12.0).abs_tolerance(0.0001))]
  pub float_tolerance: f64,
  #[proto(timestamp, validate = |v| v.gt_now().now_tolerance(Duration { seconds: 5, nanos: 0 }))]
  pub timestamp_tolerance: Option<Timestamp>,
}

#[proto_oneof(no_auto_test)]
pub enum TestOneof2 {
  #[proto(tag = 1)]
  String(String),
  #[proto(tag = 2, validate = |v| v.const_(1))]
  Number(i32),
}

#[proto_message(no_auto_test)]
pub struct DefaultValidatorTest2 {
  #[proto(message)]
  pub msg_with_default_validator: Option<DefaultValidatorTest>,
}

#[allow(clippy::use_self)]
#[proto_message(no_auto_test)]
#[proto(cel_rules(cel_program!(id = "id_is_1", msg = "abc", expr = "this.id == 1")))]
pub struct DefaultValidatorTest {
  pub id: i32,
  #[proto(oneof(required, tags(1, 2)))]
  pub test_oneof2: Option<TestOneof2>,
  #[proto(repeated(message))]
  pub repeated_test: Vec<DefaultValidatorTest>,
  #[proto(map(int32, message))]
  pub map_test: HashMap<i32, DefaultValidatorTest>,
}

#[proto_oneof(no_auto_test)]
pub enum TestOneof {
  #[proto(tag = 1, validate = |v| v.cel(cel_program!(id = "string_cel_rule", msg = "abc", expr = "this != 'b'")))]
  String(String),
  #[proto(tag = 2, message(boxed), validate = |v| v.cel(cel_program!(id = "recursive_cel_rule", msg = "abc", expr = "this.string != 'c'")))]
  BoxedMsg(Box<OneofTests>),
  #[proto(tag = 3, message)]
  DefaultValidatorMsg(DefaultValidatorTest),
}

#[proto_message(no_auto_test)]
pub struct OneofTests {
  #[proto(oneof(tags(1, 2, 3)))]
  pub test_oneof: Option<TestOneof>,
}

#[proto_message(no_auto_test)]
pub struct MapTests {
  #[proto(map(int32, int32), validate = |v| v.min_pairs(1))]
  pub min_pairs_test: HashMap<i32, i32>,
  #[proto(map(int32, int32), validate = |v| v.max_pairs(1))]
  pub max_pairs_test: HashMap<i32, i32>,
  #[proto(map(int32, int32), validate = |v| v.keys(|k| k.gte(2).cel(cel_program!(id = "key_rule", msg = "abc", expr = "this <= 15"))))]
  pub keys_test: HashMap<i32, i32>,
  #[proto(map(int32, int32), validate = |v| v.values(|vals| vals.gte(2).cel(cel_program!(id = "value_rule", msg = "abc", expr = "this <= 15"))))]
  pub values_test: HashMap<i32, i32>,
  #[proto(map(int32, int32), validate = |v| v.cel(cel_program!(id = "cel_rule", msg = "abc", expr = "1 in this && this[1] == 1")).ignore_if_zero_value())]
  pub cel_test: HashMap<i32, i32>,
}

#[proto_message(no_auto_test)]
pub struct RepeatedTests {
  #[proto(repeated(int32), validate = |v| v.items(|i| i.const_(15)))]
  pub items_test: Vec<i32>,
  #[proto(repeated(int32), validate = |v| v.items(|i| i.cel(cel_program!(id = "cel_rule", msg = "abc", expr = "this == 1"))))]
  pub items_cel_test: Vec<i32>,
  #[proto(repeated(int32), validate = |v| v.cel(cel_program!(id = "cel_rule", msg = "abc", expr = "this[0] == 1")).ignore_if_zero_value())]
  pub cel_test: Vec<i32>,
}

#[proto_message(no_auto_test)]
pub struct DummyMsg {
  #[proto(tag = 1)]
  pub id: i32,
}

#[proto_enum]
pub enum DummyEnum {
  A,
  B,
  C,
}

#[proto_message(no_auto_test)]
pub struct UniqueEnums {
  #[proto(repeated(enum_(DummyEnum)), tag = 1, validate = |v| v.unique())]
  pub unique_enums: Vec<i32>,
}

#[proto_message(no_auto_test)]
pub struct UniqueFloats {
  #[proto(tag = 1, validate = |v| v.unique())]
  pub unique_floats: Vec<f32>,
}

#[proto_message(no_auto_test)]
pub struct UniqueMessages {
  #[proto(repeated(message), tag = 1, validate = |v| v.unique())]
  pub unique_messages: Vec<DummyMsg>,
}

#[proto_message(no_auto_test)]
pub struct UniqueBytes {
  #[proto(repeated(message), tag = 1, validate = |v| v.unique())]
  pub unique_bytes: Vec<Bytes>,
}

#[proto_message(no_auto_test)]
pub struct MinItems {
  #[proto(repeated(int32), tag = 1, validate = |v| v.min_items(3))]
  pub items: Vec<i32>,
}

#[proto_message(no_auto_test)]
pub struct MaxItems {
  #[proto(repeated(int32), tag = 1, validate = |v| v.max_items(1))]
  pub items: Vec<i32>,
}

#[proto_enum]
pub enum TestEnum {
  Unspecified = 0,
  One = 1,
  Two = 2,
}

#[proto_message(no_auto_test)]
pub struct EnumRules {
  #[proto(enum_(TestEnum), validate = |v| v.const_(1))]
  pub const_test: i32,
  #[proto(enum_(TestEnum), validate = |v| v.in_([1]))]
  pub in_test: i32,
  #[proto(enum_(TestEnum), validate = |v| v.not_in([1]).ignore_if_zero_value())]
  pub not_in_test: i32,
  #[proto(enum_(TestEnum), validate = |v| v.defined_only())]
  pub defined_only_test: i32,
  #[proto(enum_(TestEnum), validate = |v| v.cel(cel_program!(id = "cel_rule", msg = "abc", expr = "this == 1")))]
  pub cel_test: i32,
  #[proto(enum_(TestEnum), validate = |v| v.required())]
  pub required_test: Option<i32>,
  #[proto(enum_(TestEnum), validate = |v| v.not_in([1]).ignore_always())]
  pub ignore_always_test: i32,
}

#[proto_message(no_auto_test)]
pub struct FieldMaskRules {
  #[proto(field_mask, validate = |v| v.const_(["tom_bombadil"]))]
  pub const_test: Option<FieldMask>,
  #[proto(field_mask, validate = |v| v.in_(["tom_bombadil"]))]
  pub in_test: Option<FieldMask>,
  #[proto(field_mask, validate = |v| v.not_in(["tom_bombadil"]))]
  pub not_in_test: Option<FieldMask>,
  #[proto(field_mask, validate = |v| v.cel(cel_program!(id = "cel_rule", msg = "abc", expr = "this.paths[0] == 'tom_bombadil'")))]
  pub cel_test: Option<FieldMask>,
  #[proto(field_mask, validate = |v| v.required())]
  pub required_test: Option<FieldMask>,
  #[proto(field_mask, validate = |v| v.not_in(["tom_bombadil"]).ignore_always())]
  pub ignore_always_test: Option<FieldMask>,
}

#[proto_message(no_auto_test)]
pub struct AnyRules {
  #[proto(any, validate = |v| v.in_(["/type_url"]))]
  pub in_test: Option<Any>,
  #[proto(any, validate = |v| v.not_in(["/type_url"]))]
  pub not_in_test: Option<Any>,
  #[proto(any, validate = |v| v.cel(cel_program!(id = "cel_rule", msg = "abc", expr = "this.value == b'a'")))]
  pub cel_test: Option<Any>,
  #[proto(any, validate = |v| v.required())]
  pub required_test: Option<Any>,
  #[proto(any, validate = |v| v.not_in(["/type_url"]).ignore_always())]
  pub ignore_always_test: Option<Any>,
}

#[proto_message(no_auto_test)]
pub struct TimestampRules {
  #[proto(timestamp, validate = |v| v.const_(Timestamp::default()))]
  pub const_test: Option<Timestamp>,
  #[proto(timestamp, validate = |v| v.lt(Timestamp::default()))]
  pub lt_test: Option<Timestamp>,
  #[proto(timestamp, validate = |v| v.lte(Timestamp::default()))]
  pub lte_test: Option<Timestamp>,
  #[proto(timestamp, validate = |v| v.gt(Timestamp::default()))]
  pub gt_test: Option<Timestamp>,
  #[proto(timestamp, validate = |v| v.gte(Timestamp::default()))]
  pub gte_test: Option<Timestamp>,
  #[proto(timestamp, validate = |v| v.within(Duration { seconds: 10, nanos: 0 }))]
  pub within_test: Option<Timestamp>,
  #[proto(timestamp, validate = |v| v.lt_now())]
  pub lt_now_test: Option<Timestamp>,
  #[proto(timestamp, validate = |v| v.gt_now())]
  pub gt_now_test: Option<Timestamp>,
  #[proto(timestamp, validate = |v| v.required())]
  pub required_test: Option<Timestamp>,
  #[proto(timestamp, validate = |v| v.const_(Timestamp::default()).ignore_always())]
  pub ignore_always_test: Option<Timestamp>,
  #[proto(timestamp, validate = |v| v.cel(cel_program!(id = "cel_rule", msg = "abc", expr = "this < timestamp('2024-01-01T00:00:00Z')")))]
  pub cel_test: Option<Timestamp>,
}

#[proto_message(no_auto_test)]
pub struct DurationRules {
  #[proto(duration, validate = |v| v.const_(Duration::default()))]
  pub const_test: Option<Duration>,
  #[proto(duration, validate = |v| v.lt(Duration::default()))]
  pub lt_test: Option<Duration>,
  #[proto(duration, validate = |v| v.lte(Duration::default()))]
  pub lte_test: Option<Duration>,
  #[proto(duration, validate = |v| v.gt(Duration::default()))]
  pub gt_test: Option<Duration>,
  #[proto(duration, validate = |v| v.gte(Duration::default()))]
  pub gte_test: Option<Duration>,
  #[proto(duration, validate = |v| v.in_([ Duration::default() ]))]
  pub in_test: Option<Duration>,
  #[proto(duration, validate = |v| v.not_in([ Duration::default() ]))]
  pub not_in_test: Option<Duration>,
  #[proto(duration, validate = |v| v.required())]
  pub required_test: Option<Duration>,
  #[proto(duration, validate = |v| v.const_(Duration::default()).ignore_always())]
  pub ignore_always_test: Option<Duration>,
  #[proto(duration, validate = |v| v.cel(cel_program!(id = "cel_rule", msg = "abc", expr = "this < duration('5m')")))]
  pub cel_test: Option<Duration>,
}

#[proto_message(no_auto_test)]
pub struct BytesRules {
  #[proto(validate = |v| v.const_(b"a"))]
  pub const_test: Bytes,
  #[proto(validate = |v| v.len(1))]
  pub len_test: Bytes,
  #[proto(validate = |v| v.min_len(1))]
  pub min_len_test: Bytes,
  #[proto(validate = |v| v.max_len(1))]
  pub max_len_test: Bytes,
  #[proto(validate = |v| v.pattern("a"))]
  pub pattern_test: Bytes,
  #[proto(validate = |v| v.prefix(b"a"))]
  pub prefix_test: Bytes,
  #[proto(validate = |v| v.suffix(b"a"))]
  pub suffix_test: Bytes,
  #[proto(validate = |v| v.contains(b"a"))]
  pub contains_test: Bytes,
  #[proto(validate = |v| v.ip())]
  pub ip_test: Bytes,
  #[proto(validate = |v| v.ipv4())]
  pub ipv4_test: Bytes,
  #[proto(validate = |v| v.ipv6())]
  pub ipv6_test: Bytes,
  #[proto(validate = |v| v.uuid())]
  pub uuid_test: Bytes,
  #[proto(validate = |v| v.cel(cel_program!(id = "cel_rule", msg = "abc", expr = "this == b'a'")))]
  pub cel_test: Bytes,
  #[proto(validate = |v| v.required())]
  pub required_test: Option<Bytes>,
  #[proto(validate = |v| v.const_(b"a").ignore_if_zero_value())]
  pub ignore_if_zero_value_test: Option<Bytes>,
  #[proto(validate = |v| v.const_(b"b").ignore_always())]
  pub ignore_always_test: Bytes,
}

#[proto_message(no_auto_test)]
pub struct BoolRules {
  #[proto(validate = |v| v.const_(true))]
  pub const_test: bool,
  #[proto(validate = |v| v.required())]
  pub required_test: Option<bool>,
  #[proto(validate = |v| v.const_(true).ignore_if_zero_value())]
  pub ignore_if_zero_value_test: Option<bool>,
  #[proto(validate = |v| v.const_(true).ignore_always())]
  pub ignore_always_test: bool,
}

macro_rules! string_rules {
  ($($well_known:ident),*) => {
    paste::paste! {
      #[proto_message(no_auto_test)]
      pub struct StringRules {
        #[proto(validate = |v| v.const_("a"))]
        pub const_test: String,
        #[proto(validate = |v| v.len(1))]
        pub len_test: String,
        #[proto(validate = |v| v.min_len(1))]
        pub min_len_test: String,
        #[proto(validate = |v| v.max_len(1))]
        pub max_len_test: String,
        #[proto(validate = |v| v.len_bytes(1))]
        pub len_bytes_test: String,
        #[proto(validate = |v| v.min_bytes(1))]
        pub min_bytes_test: String,
        #[proto(validate = |v| v.max_bytes(1))]
        pub max_bytes_test: String,
        #[proto(validate = |v| v.pattern("a"))]
        pub pattern_test: String,
        #[proto(validate = |v| v.prefix("a"))]
        pub prefix_test: String,
        #[proto(validate = |v| v.suffix("a"))]
        pub suffix_test: String,
        #[proto(validate = |v| v.contains("a"))]
        pub contains_test: String,
        #[proto(validate = |v| v.not_contains("a"))]
        pub not_contains_test: String,
        #[proto(validate = |v| v.cel(cel_program!(id = "cel_rule", msg = "abc", expr = "this == 'a'")))]
        pub cel_test: String,
        #[proto(validate = |v| v.required())]
        pub required_test: Option<String>,
        #[proto(validate = |v| v.const_("a").ignore_if_zero_value())]
        pub ignore_if_zero_value_test: Option<String>,
        #[proto(validate = |v| v.const_("b").ignore_always())]
        pub ignore_always_test: String,
        $(
          #[proto(validate = |v| v.$well_known())]
          pub [< $well_known _test >]: String,
        )*
      }
    }
  };
}

string_rules!(
  email,
  hostname,
  ip,
  ipv4,
  ipv6,
  uri,
  uri_ref,
  address,
  ulid,
  uuid,
  tuuid,
  ip_with_prefixlen,
  ipv4_with_prefixlen,
  ipv6_with_prefixlen,
  ip_prefix,
  ipv4_prefix,
  ipv6_prefix,
  host_and_port,
  header_name_strict,
  header_name_loose,
  header_value_strict,
  header_value_loose
);

macro_rules! impl_numeric {
  ($name:ident, $typ:ty $(, $finite:ident)?) => {
    macro_rules! num {
      (finite) => (1.0);
      () => (1);
    }

    paste::paste! {
      #[allow(unused, clippy::struct_field_names)]
      #[proto_message(no_auto_test)]
      pub struct [< $name:camel Rules >] {
        #[proto($name, validate = |v| v.required())]
        pub required_test: Option<$typ>,
        #[proto($name, validate = |v| v.lt(num!($($finite)?)))]
        pub lt_test: $typ,
        #[proto($name, validate = |v| v.lte(num!($($finite)?)))]
        pub lte_test: $typ,
        #[proto($name, validate = |v| v.gt(num!($($finite)?)))]
        pub gt_test: $typ,
        #[proto($name, validate = |v| v.gte(num!($($finite)?)))]
        pub gte_test: $typ,
        #[proto($name, validate = |v| v.in_([num!($($finite)?)]))]
        pub in_test: $typ,
        #[proto($name, validate = |v| v.not_in([num!($($finite)?)]))]
        pub not_in_test: $typ,
        #[proto($name, validate = |v| v.cel(cel_program!(id = "cel_rule", msg = "abc", expr = "this != 0")))]
        pub cel_test: $typ,
        #[proto($name, validate = |v| v.const_(num!($($finite)?)))]
        pub const_test: $typ,
        #[proto($name, validate = |v| v.const_(num!($($finite)?)).ignore_if_zero_value())]
        pub ignore_if_zero_value_test: Option<$typ>,
        #[proto($name, validate = |v| v.const_(num!($($finite)?)).ignore_always())]
        pub ignore_always_test: $typ,
        $(
          #[proto($name, validate = |v| v.$finite())]
          pub finite_test: $typ,
        )?
      }
    }
  };
}

impl_numeric!(int64, i64);
impl_numeric!(sint64, i64);
impl_numeric!(sfixed64, i64);
impl_numeric!(int32, i32);
impl_numeric!(sint32, i32);
impl_numeric!(sfixed32, i32);
impl_numeric!(uint64, u64);
impl_numeric!(uint32, u32);
impl_numeric!(fixed64, u64);
impl_numeric!(fixed32, u32);
impl_numeric!(double, f64, finite);
impl_numeric!(float, f32, finite);
