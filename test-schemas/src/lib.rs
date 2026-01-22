#![allow(clippy::struct_field_names, clippy::must_use_candidate)]

#[cfg(test)]
mod tests {
  use super::*;
  use similar_asserts::assert_eq as assert_eq_pretty;

  mod consistency_tests;
}

use ::bytes::Bytes;
use prelude::{proto_enum, proto_oneof};
use prelude::{
  proto_types::{Any, Duration, FieldMask, Timestamp},
  *,
};
use std::collections::HashMap;

proto_package!(TEST_SCHEMAS, name = "test_schemas.v1", no_cel_test);
define_proto_file!(
  TEST_SCHEMAS_FILE,
  name = "test_schemas.proto",
  package = TEST_SCHEMAS
);

#[proto_oneof(no_auto_test)]
pub enum SimpleOneof {
  #[proto(tag = 1, validate = |v| v.const_(1))]
  A(i32),
  #[proto(tag = 2)]
  B(u32),
}

#[proto_message(no_auto_test)]
pub struct SimpleMsg {
  #[proto(validate = |v| v.const_(1))]
  pub id: i32,
  #[proto(validate = |v| v.min_len(2))]
  pub name: String,
}

#[proto_message(no_auto_test)]
pub struct FailFastTest {
  #[proto(validate = |v| v.max_len(1).not_in(["abc"]))]
  pub string: String,
  #[proto(bytes, validate = |v| v.max_len(1).not_in([b"abc"]))]
  pub bytes: Bytes,
  #[proto(validate = |v| v.gt(1).not_in([1]))]
  pub int: i32,
  #[proto(validate = |v| v.gt(1.0).not_in([1.0]))]
  pub float: f32,
  #[proto(duration, validate = |v| v.gt(Duration::default()).not_in([Duration::default()]))]
  pub duration: Option<Duration>,
  #[proto(timestamp, validate = |v| v.gt_now().within(Duration::new(10, 0)))]
  pub timestamp: Option<Timestamp>,
  #[proto(field_mask, validate = |v| v.in_(["abc"]).not_in(["abcde"]))]
  pub field_mask: Option<FieldMask>,
  #[proto(enum_(TestEnum), validate = |v| v.defined_only().not_in([45]))]
  pub enum_field: i32,
  #[proto(oneof(tags(1, 2)))]
  pub simple_oneof: Option<SimpleOneof>,
  #[proto(message)]
  pub message: Option<SimpleMsg>,
}

#[proto_message(no_auto_test)]
pub struct ConstRulesTest {
  #[proto(validate = |v| v.const_("abc").min_len(3))]
  pub string: String,
  #[proto(bytes, validate = |v| v.const_(b"abc").min_len(3))]
  pub bytes: Bytes,
  #[proto(validate = |v| v.const_(3).gt(2))]
  pub int: i32,
  #[proto(validate = |v| v.const_(3.0).gt(2.0))]
  pub float: f32,
  #[proto(duration, validate = |v| v.const_(Duration::new(1, 0)).gt(Duration::default()))]
  pub duration: Option<Duration>,
  #[proto(timestamp, validate = |v| v.const_(Timestamp::new(1, 0)).gt(Timestamp::default()))]
  pub timestamp: Option<Timestamp>,
  #[proto(field_mask, validate = |v| v.const_(["abc"]).in_(["abc"]))]
  pub field_mask: Option<FieldMask>,
  #[proto(enum_(TestEnum), validate = |v| v.const_(1).defined_only())]
  pub enum_field: i32,
}

// Placing it here so I can check if reflection for these works fine
#[proto_message(no_auto_test)]
pub struct RustKeywords {
  pub r#as: String,
  pub r#break: String,
  pub r#const: String,
  pub r#continue: String,
  pub r#else: String,
  pub r#enum: String,
  pub r#false: String,
  pub r#fn: String,
  pub r#for: String,
  pub r#if: String,
  pub r#impl: String,
  pub r#in: String,
  pub r#let: String,
  pub r#loop: String,
  pub r#match: String,
  pub r#mod: String,
  pub r#move: String,
  pub r#mut: String,
  pub r#pub: String,
  pub r#ref: String,
  pub r#return: String,
  pub r#static: String,
  pub r#struct: String,
  pub r#trait: String,
  pub r#true: String,
  pub r#type: String,
  pub r#unsafe: String,
  pub r#use: String,
  pub r#where: String,
  pub r#while: String,
  pub r#abstract: String,
  pub r#become: String,
  pub r#box: String,
  pub r#do: String,
  pub r#final: String,
  pub r#macro: String,
  pub r#override: String,
  pub r#priv: String,
  pub r#typeof: String,
  pub r#unsized: String,
  pub r#virtual: String,
  pub r#yield: String,
  pub r#try: String,
  pub r#async: String,
  pub r#await: String,
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_keywords() {
    let schema = RustKeywords::proto_schema();

    let keywords = [
      "as", "break", "const", "continue", "else", "enum", "false", "fn", "for", "if", "impl", "in",
      "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "static", "struct",
      "trait", "true", "type", "unsafe", "use", "where", "while", "abstract", "become", "box",
      "do", "final", "macro", "override", "priv", "typeof", "unsized", "virtual", "yield", "try",
      "async", "await",
    ];

    for (field, exp_name) in schema.fields().zip(keywords) {
      assert_eq!(field.name, exp_name);
    }
  }
}

#[proto_message(no_auto_test)]
pub struct BTreeMapTest {
  #[proto(map(int32, int32), validate = |v| v.min_pairs(1).max_pairs(2))]
  pub map: BTreeMap<i32, i32>,
}

fn bad_rule() -> CelProgram {
  cel_program!(id = "abc", msg = "hi", expr = "hi")
}

#[allow(clippy::use_self)]
#[proto_message(no_auto_test)]
pub struct BoxedMsg {
  #[proto(message)]
  pub msg: Option<Box<BoxedMsg>>,
  #[proto(validate = |v| v.const_(1))]
  pub id: i32,
}

#[proto_message(no_auto_test)]
pub struct BadFieldRules {
  #[proto(tag = 1, validate = |v| v.cel(bad_rule()))]
  pub id: i32,
}

#[proto_message(no_auto_test)]
#[proto(validate = |v| v.cel(bad_rule()))]
pub struct BadMsgRules {
  #[proto(tag = 1)]
  pub id: i32,
}

// Just to let the oneof be picked up in the schema
#[proto_message(no_auto_test)]
pub struct BadCelOneofTest {
  #[proto(oneof(tags(1, 2)))]
  pub bad_cel_oneof: Option<BadCelOneof>,
}

#[proto_oneof(no_auto_test)]
pub enum BadCelOneof {
  #[proto(tag = 1, validate = |v| v.cel(bad_rule()))]
  Id(i32),
  #[proto(tag = 2)]
  Name(String),
}

// This checks if the validator is registered even if there are no
// validators explicitely defined, but a field is a message
#[proto_message(no_auto_test)]
pub struct DefaultValidatorTestMsg {
  #[proto(message)]
  pub msg_with_default_validator: Option<DefaultValidatorTestCel>,
}

// This checks if the default validator is registered
// if a field is a oneof
#[proto_message(no_auto_test)]
pub struct DefaultValidatorTestOneof {
  #[proto(oneof(required, tags(1, 2)))]
  pub default_validator_oneof: Option<DefaultValidatorOneof>,
}

// This checks if the default validator is registered
// if a variant is a message
#[proto_oneof(no_auto_test)]
pub enum DefaultValidatorOneof {
  #[proto(message, tag = 1)]
  A(SimpleMsg),
  #[proto(tag = 2)]
  B(u32),
}

// Checks if the default validator is registered if there is a
// repeated message field
#[proto_message(no_auto_test)]
pub struct DefaultValidatorTestVec {
  #[proto(repeated(message))]
  pub repeated_test: Vec<DefaultValidatorTestCel>,
}

// Checks if the default validator is registered if there is a
// map field with message values
#[proto_message(no_auto_test)]
pub struct DefaultValidatorTestMap {
  #[proto(map(int32, message))]
  pub map_test: HashMap<i32, DefaultValidatorTestCel>,
}

// This checks if the default validator is registered
// if there are top level rules
#[allow(clippy::use_self)]
#[proto_message(no_auto_test)]
#[proto(validate = |v| v.cel(cel_program!(id = "id_is_1", msg = "abc", expr = "this.id == 1")))]
pub struct DefaultValidatorTestCel {
  pub id: i32,
}

#[proto_oneof(no_auto_test)]
pub enum TestOneof {
  #[proto(tag = 1, validate = |v| v.cel(cel_program!(id = "string_cel_rule", msg = "abc", expr = "this != 'b'")))]
  String(String),
  #[proto(tag = 2, message(boxed), validate = |v| v.cel(cel_program!(id = "recursive_cel_rule", msg = "abc", expr = "this.string != 'c'")))]
  BoxedMsg(Box<OneofTests>),
}

#[proto_message(no_auto_test)]
pub struct OneofTests {
  #[proto(oneof(tags(1, 2, 3)))]
  pub test_oneof: Option<TestOneof>,
}

#[proto_message(no_auto_test)]
pub struct MapTests {
  // Just to check that the associated types are resolving correctly for String
  #[proto(map(string, string), validate = |v| v.min_pairs(1).keys(|k| k.const_("abc")).values(|v| v.const_("abc")))]
  pub string_test: HashMap<String, String>,
  // Checking if the type wrappers work fine
  #[proto(map(sint32, sint32), validate = |v| v.min_pairs(1).keys(|k| k.const_(1)).values(|v| v.const_(1)))]
  pub wrappers_test: HashMap<i32, i32>,
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
  // Just to check that the associated types are resolving correctly for String
  #[proto(repeated(string), validate = |v| v.min_items(1).items(|i| i.const_("abc")))]
  pub string_test: Vec<String>,
  // Checking if the type wrappers work fine
  #[proto(repeated(sint32), validate = |v| v.min_items(1).items(|i| i.const_(1)))]
  pub wrappers_test: Vec<i32>,
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
  #[proto(tag = 1, validate = |v| v.unique().items(|i| i.abs_tolerance(0.0001)))]
  pub unique_floats: Vec<f32>,
}

#[proto_message(no_auto_test)]
pub struct UniqueMessages {
  #[proto(repeated(message), tag = 1, validate = |v| v.unique())]
  pub unique_messages: Vec<DummyMsg>,
}

#[proto_message(no_auto_test)]
pub struct UniqueBytes {
  #[proto(repeated(bytes), tag = 1, validate = |v| v.unique())]
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
