use crate::*;

#[derive(Debug, PartialEq)]
pub struct Extension {
  pub target: ExtensionTarget,
  pub fields: Vec<Field>,
}

pub trait ProtoExtension {
  fn as_proto_extension() -> Extension;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum ExtensionTarget {
  FileOptions,
  MessageOptions,
  FieldOptions,
  OneofOptions,
  EnumOptions,
  EnumValueOptions,
  ServiceOptions,
  MethodOptions,
}

impl ExtensionTarget {
  #[must_use]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::FileOptions => "google.protobuf.FileOptions",
      Self::MessageOptions => "google.protobuf.MessageOptions",
      Self::FieldOptions => "google.protobuf.FieldOptions",
      Self::OneofOptions => "google.protobuf.OneofOptions",
      Self::EnumOptions => "google.protobuf.EnumOptions",
      Self::EnumValueOptions => "google.protobuf.EnumValueOptions",
      Self::ServiceOptions => "google.protobuf.ServiceOptions",
      Self::MethodOptions => "google.protobuf.MethodOptions",
    }
  }
}

impl Display for ExtensionTarget {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.as_str())
  }
}
