use std::collections::HashMap;

use prelude::*;

proto_package!(TESTING_PKG, name = "testing", no_cel_test);

define_proto_file!(TESTING, name = "testing.proto", package = TESTING_PKG);

#[allow(clippy::use_self)]
#[proto_message(no_auto_test)]
struct MsgWithNoValidator {
  id: i32,
  #[proto(oneof(tags(1, 2)))]
  oneof: Option<OneofWithNoValidator>,
  #[proto(message)]
  recursive: Option<Box<MsgWithNoValidator>>,
  #[proto(repeated(message))]
  vec: Vec<MsgWithNoValidator>,
  #[proto(map(int32, message))]
  map: HashMap<i32, MsgWithNoValidator>,
}

#[proto_oneof(no_auto_test)]
enum OneofWithNoValidator {
  #[proto(tag = 1)]
  A(i32),
  #[proto(tag = 2)]
  B(i32),
}

// If the validator is being correctly detected as empty, the assembly output should be more or less like this:
//
// .section .text.trigger_validation,"ax",@progbits
// .globl  trigger_validation
// .p2align        4
// .type   trigger_validation,@function
// trigger_validation:
// .cfi_startproc
// ret

#[unsafe(no_mangle)]
#[inline(never)]
fn trigger_validation(msg: &MsgWithNoValidator) {
  let _ = msg.validate();
}

fn main() {}
