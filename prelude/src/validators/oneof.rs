use super::*;

reusable_string!(ONEOF_REQUIRED, "(buf.validate.field).required");

/// Specifies that at least one of the variants of the oneof must be set in order to pass validation.
pub fn oneof_required() -> ProtoOption {
  ProtoOption {
    name: ONEOF_REQUIRED.clone(),
    value: OptionValue::Bool(true),
  }
}
