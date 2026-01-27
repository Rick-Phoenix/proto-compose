use crate::*;
use proto_types::field_descriptor_proto::Type as DescriptorType;

pub trait AsProtoType {
  fn proto_type() -> ProtoType;
}

pub trait AsProtoField {
  fn as_proto_field() -> FieldType;
}

impl<T> AsProtoField for Option<T>
where
  T: AsProtoType,
{
  #[inline]
  fn as_proto_field() -> FieldType {
    let type_ = T::proto_type();

    if type_.is_message() {
      FieldType::Normal(type_)
    } else {
      FieldType::Optional(type_)
    }
  }
}

impl<T: AsProtoType> AsProtoField for T {
  #[inline]
  fn as_proto_field() -> FieldType {
    FieldType::Normal(T::proto_type())
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FieldType {
  Normal(ProtoType),
  Map {
    keys: ProtoMapKey,
    values: ProtoType,
  },
  Repeated(ProtoType),
  Optional(ProtoType),
}

pub(crate) struct Sealed;

pub trait AsProtoMapKey {
  fn as_proto_map_key() -> ProtoMapKey;
  #[allow(private_interfaces)]
  const SEALED: Sealed;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ProtoMapKey {
  String,
  Bool,
  Int32,
  Int64,
  Sint32,
  Sint64,
  Sfixed32,
  Sfixed64,
  Fixed32,
  Fixed64,
  Uint32,
  Uint64,
}

impl ProtoMapKey {
  #[must_use]
  pub fn into_type(self) -> ProtoType {
    self.into()
  }
}

impl From<ProtoMapKey> for ProtoType {
  fn from(value: ProtoMapKey) -> Self {
    Self::Scalar(value.into())
  }
}

impl From<ProtoMapKey> for ProtoScalar {
  fn from(value: ProtoMapKey) -> Self {
    match value {
      ProtoMapKey::String => Self::String,
      ProtoMapKey::Bool => Self::Bool,
      ProtoMapKey::Int32 => Self::Int32,
      ProtoMapKey::Int64 => Self::Int64,
      ProtoMapKey::Sint32 => Self::Sint32,
      ProtoMapKey::Sint64 => Self::Sint64,
      ProtoMapKey::Sfixed32 => Self::Sfixed32,
      ProtoMapKey::Sfixed64 => Self::Sfixed64,
      ProtoMapKey::Fixed32 => Self::Fixed32,
      ProtoMapKey::Fixed64 => Self::Fixed64,
      ProtoMapKey::Uint32 => Self::Uint32,
      ProtoMapKey::Uint64 => Self::Uint64,
    }
  }
}

impl Display for ProtoMapKey {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::Int32 => write!(f, "int32"),
      Self::Int64 => write!(f, "int64"),
      Self::Uint32 => write!(f, "uint32"),
      Self::Uint64 => write!(f, "uint64"),
      Self::Sint32 => write!(f, "sint32"),
      Self::Sint64 => write!(f, "sint64"),
      Self::Fixed32 => write!(f, "fixed32"),
      Self::Fixed64 => write!(f, "fixed64"),
      Self::Sfixed32 => write!(f, "sfixed32"),
      Self::Sfixed64 => write!(f, "sfixed64"),
      Self::Bool => write!(f, "bool"),
      Self::String => write!(f, "string"),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ProtoScalar {
  Double,
  Float,
  Int32,
  Int64,
  Uint32,
  Uint64,
  Sint32,
  Sint64,
  Fixed32,
  Fixed64,
  Sfixed32,
  Sfixed64,
  Bool,
  String,
  Bytes,
}

impl From<ProtoScalar> for DescriptorType {
  #[inline]
  fn from(value: ProtoScalar) -> Self {
    match value {
      ProtoScalar::Double => Self::Double,
      ProtoScalar::Float => Self::Float,
      ProtoScalar::Int32 => Self::Int32,
      ProtoScalar::Int64 => Self::Int64,
      ProtoScalar::Uint32 => Self::Uint32,
      ProtoScalar::Uint64 => Self::Uint64,
      ProtoScalar::Sint32 => Self::Sint32,
      ProtoScalar::Sint64 => Self::Sint64,
      ProtoScalar::Fixed32 => Self::Fixed32,
      ProtoScalar::Fixed64 => Self::Fixed64,
      ProtoScalar::Sfixed32 => Self::Sfixed32,
      ProtoScalar::Sfixed64 => Self::Sfixed64,
      ProtoScalar::Bool => Self::Bool,
      ProtoScalar::String => Self::String,
      ProtoScalar::Bytes => Self::Bytes,
    }
  }
}

impl Display for ProtoScalar {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::Double => write!(f, "double"),
      Self::Float => write!(f, "float"),
      Self::Int32 => write!(f, "int32"),
      Self::Int64 => write!(f, "int64"),
      Self::Uint32 => write!(f, "uint32"),
      Self::Uint64 => write!(f, "uint64"),
      Self::Sint32 => write!(f, "sint32"),
      Self::Sint64 => write!(f, "sint64"),
      Self::Fixed32 => write!(f, "fixed32"),
      Self::Fixed64 => write!(f, "fixed64"),
      Self::Sfixed32 => write!(f, "sfixed32"),
      Self::Sfixed64 => write!(f, "sfixed64"),
      Self::Bool => write!(f, "bool"),
      Self::String => write!(f, "string"),
      Self::Bytes => write!(f, "bytes"),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ProtoType {
  Scalar(ProtoScalar),
  Message(ProtoPath),
  Enum(ProtoPath),
}

impl From<ProtoType> for DescriptorType {
  #[inline]
  fn from(value: ProtoType) -> Self {
    match value {
      ProtoType::Scalar(scalar) => scalar.into(),
      ProtoType::Message(_) => Self::Message,
      ProtoType::Enum(_) => Self::Enum,
    }
  }
}

impl Display for ProtoType {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::Scalar(proto_scalar) => write!(f, "{proto_scalar}"),
      Self::Message(proto_path) | Self::Enum(proto_path) => write!(f, "{proto_path}"),
    }
  }
}

impl ProtoType {
  /// Returns `true` if the proto type is [`Message`].
  ///
  /// [`Message`]: ProtoType::Message
  #[must_use]
  pub const fn is_message(&self) -> bool {
    matches!(self, Self::Message { .. })
  }

  /// Returns `true` if the proto type is [`Enum`].
  ///
  /// [`Enum`]: ProtoType::Enum
  #[must_use]
  pub const fn is_enum(&self) -> bool {
    matches!(self, Self::Enum(..))
  }
}

impl FieldType {
  pub(crate) fn render(&self, current_package: &FixedStr) -> Cow<'_, str> {
    let name = self.render_name(current_package);

    match self {
      Self::Normal(_) | Self::Map { .. } => name,
      Self::Repeated(_) => format!("repeated {name}").into(),
      Self::Optional(inner) => {
        if inner.is_message() {
          name
        } else {
          format!("optional {name}").into()
        }
      }
    }
  }

  pub(crate) fn render_name(&self, current_package: &FixedStr) -> Cow<'_, str> {
    match self {
      Self::Normal(type_info) | Self::Repeated(type_info) | Self::Optional(type_info) => {
        type_info.render_name(current_package)
      }
      Self::Map { keys, values } => format!(
        "map<{}, {}>",
        keys.into_type().render_name(current_package),
        values.render_name(current_package)
      )
      .into(),
    }
  }
}

impl ProtoType {
  pub(crate) fn render_name(&self, current_package: &FixedStr) -> Cow<'_, str> {
    match self {
      Self::Scalar(scalar) => scalar.to_string().into(),
      Self::Message(path) | Self::Enum(path) => {
        if *path.package == **current_package {
          path.name.as_ref().into()
        } else {
          format!("{}.{}", path.package, path.name).into()
        }
      }
    }
  }

  pub(crate) fn register_import(&self, imports: &mut FileImports) {
    match self {
      Self::Scalar { .. } => {}
      Self::Message(path) | Self::Enum(path) => imports.insert_from_path(path),
    }
  }
}

impl ProtoPath {
  pub(crate) fn render_name(&self, current_package: &FixedStr) -> Cow<'_, str> {
    if self.package == *current_package {
      self.name.as_ref().into()
    } else {
      format!("{}.{}", self.package, self.name).into()
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProtoPath {
  pub name: FixedStr,
  pub package: FixedStr,
  pub file: FixedStr,
}

impl Display for ProtoPath {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    let Self { name, package, .. } = self;

    write!(f, "{package}.{name}")
  }
}
