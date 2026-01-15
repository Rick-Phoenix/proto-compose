#![no_std]

#[cfg(any(test, feature = "std"))]
extern crate std;

use prelude::BTreeMap;

use prelude::{ValidatedMessage, define_proto_file, proto_message, proto_package};

proto_package!(PKG, name = "no_std_package");
define_proto_file!(FILE, name = "file.proto", package = PKG);

#[proto_message(no_auto_test)]
pub struct TestMsg {
  #[proto(map(int32, int32), validate = |v| v.min_pairs(2))]
  map: BTreeMap<i32, i32>,
}

fn main() {
  let msg = TestMsg::default();

  assert!(msg.validate().is_err());
}
