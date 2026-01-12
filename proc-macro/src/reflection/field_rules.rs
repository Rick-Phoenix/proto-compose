use ::proto_types::protovalidate::*;

use super::*;

impl RulesCtx {
  pub fn get_field_validator(&self, proto_type: &ProtoType) -> BuilderTokens {
    match proto_type {
      ProtoType::String => self.get_string_validator(),
      ProtoType::Bool => self.get_bool_validator(),
      ProtoType::Bytes => self.get_bytes_validator(),
      ProtoType::Enum(path) => self.get_enum_validator(path),
      ProtoType::Message(message_info) => self.get_message_field_validator(&message_info.path),
      ProtoType::Float => self.get_numeric_validator::<FloatRules>(),
      ProtoType::Double => self.get_numeric_validator::<DoubleRules>(),
      ProtoType::Int32 => self.get_numeric_validator::<Int32Rules>(),
      ProtoType::Int64 => self.get_numeric_validator::<Int64Rules>(),
      ProtoType::Uint32 => self.get_numeric_validator::<UInt32Rules>(),
      ProtoType::Uint64 => self.get_numeric_validator::<UInt64Rules>(),
      ProtoType::Sint32 => self.get_numeric_validator::<SInt32Rules>(),
      ProtoType::Sint64 => self.get_numeric_validator::<SInt64Rules>(),
      ProtoType::Fixed32 => self.get_numeric_validator::<Fixed32Rules>(),
      ProtoType::Fixed64 => self.get_numeric_validator::<Fixed64Rules>(),
      ProtoType::Sfixed32 => self.get_numeric_validator::<SFixed32Rules>(),
      ProtoType::Sfixed64 => self.get_numeric_validator::<SFixed64Rules>(),
      ProtoType::Duration => self.get_duration_validator(),
      ProtoType::Timestamp => self.get_timestamp_validator(),
      ProtoType::Any => self.get_any_validator(),
      ProtoType::FieldMask => self.get_field_mask_validator(),
    }
  }
}
