use crate::*;

reusable_string!(PROTO_DEPRECATED, "deprecated");

pub struct DEPRECATED;

impl From<DEPRECATED> for ProtoOption {
  fn from(_: DEPRECATED) -> Self {
    ProtoOption {
      name: PROTO_DEPRECATED.clone(),
      value: OptionValue::Bool(true),
    }
  }
}
