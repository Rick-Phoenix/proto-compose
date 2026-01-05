#![allow(clippy::struct_field_names)]

use bytes::Bytes;
use prelude::{
  cel_program, define_proto_file, proto_message, proto_package,
  proto_types::{Duration, Timestamp},
};

#[proto_message(no_auto_test)]
struct TimestampRules {
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
}

#[proto_message(no_auto_test)]
struct DurationRules {
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
}

#[proto_message(no_auto_test)]
struct BytesRules {
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
struct BoolRules {
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
      struct StringRules {
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

proto_package!(REFLECTION, name = "reflection.v1", no_cel_test);
define_proto_file!(
  REFLECTION_FILE,
  file = "reflection.proto",
  package = REFLECTION
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
      struct [< $name:camel Rules >] {
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

fn main() {
  REFLECTION
    .get_package()
    .render_files(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/../test-reflection/proto"
    ))
    .unwrap()
}
