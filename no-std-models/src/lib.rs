#![no_std]

extern crate alloc;
#[cfg(test)]
extern crate std;

use alloc::boxed::Box;
use alloc::string::String;

use prelude::{BTreeMap, proto_enum, proto_extension, proto_oneof, proto_service};

use prelude::{define_proto_file, proto_message, proto_package};

proto_package!(NO_STD_PKG, name = "no_std_models", no_cel_test);
define_proto_file!(FILE, name = "no_std_models.proto", package = NO_STD_PKG);

#[proto_service]
pub enum TestService {
  Service1 { request: TestMsg, response: TestMsg },
}

#[proto_extension(target = MessageOptions)]
pub struct TestExtension {
  #[proto(tag = 5000)]
  name: String,
}

#[proto_message]
pub struct TestMsg {
  #[proto(map(int32, int32), validate = |v| v.min_pairs(2))]
  pub map: BTreeMap<i32, i32>,
  #[proto(oneof(tags(1, 2)))]
  pub test_oneof: Option<TestOneof>,
  #[proto(enum_(TestEnum), validate = |v| v.defined_only())]
  pub enum_field: i32,
}

#[proto_oneof]
pub enum TestOneof {
  #[proto(tag = 1, validate = |v| v.const_(1))]
  A(i32),
  #[proto(tag = 2, validate = |v| v.const_(1))]
  B(u32),
}

#[proto_enum]
pub enum TestEnum {
  Unspecified,
  A,
  B,
}

#[proto_oneof(proxied)]
pub enum ProxiedOneofTest {
  #[proto(tag = 1, validate = |v| v.const_(1))]
  A(i32),
  #[proto(tag = 2, validate = |v| v.const_(1))]
  B(u32),
}

impl Default for ProxiedOneofTestProto {
  fn default() -> Self {
    Self::A(1)
  }
}

#[proto_message(proxied)]
pub struct ProxiedMsg {
  #[proto(map(int32, int32), validate = |v| v.min_pairs(2))]
  map: BTreeMap<i32, i32>,
  #[proto(oneof(default, proxied, tags(1, 2)))]
  proxied_oneof_test: ProxiedOneofTest,
  #[proto(enum_(TestEnum), validate = |v| v.defined_only())]
  enum_field: i32,
  #[proto(message(proxied, default))]
  recursive: Box<ProxiedMsg>,
}
