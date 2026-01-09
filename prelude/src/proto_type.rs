use std::borrow::Cow;

use crate::*;

pub trait AsProtoType {
  fn proto_type() -> ProtoType;
}

pub trait AsProtoField {
  fn as_proto_field() -> FieldType;
}

impl<T> AsProtoField for Option<T>
where
  T: AsProtoField,
{
  #[inline]
  fn as_proto_field() -> FieldType {
    match T::as_proto_field() {
      FieldType::Normal(typ) => {
        if typ.is_message() {
          FieldType::Normal(typ)
        } else {
          FieldType::Optional(typ)
        }
      }
      FieldType::Map { .. } => {
        panic!("Optional fields cannot be maps")
      }
      FieldType::Repeated(_) => {
        panic!("Optional fields cannot be repeated")
      }
      FieldType::Optional(_) => {
        panic!("Optional fields cannot be nested")
      }
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
pub enum FieldType {
  Normal(ProtoType),
  Map { keys: ProtoType, values: ProtoType },
  Repeated(ProtoType),
  Optional(ProtoType),
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
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

impl Display for ProtoScalar {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
pub enum ProtoType {
  Scalar(ProtoScalar),
  Message(ProtoPath),
  Enum(ProtoPath),
}

impl Display for ProtoType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
  pub(crate) fn render(&self, current_package: &'static str) -> Cow<'static, str> {
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

  pub(crate) fn render_name(&self, current_package: &'static str) -> Cow<'static, str> {
    match self {
      Self::Normal(type_info) | Self::Repeated(type_info) | Self::Optional(type_info) => {
        type_info.render_name(current_package)
      }
      Self::Map { keys, values } => format!(
        "map<{}, {}>",
        keys.render_name(current_package),
        values.render_name(current_package)
      )
      .into(),
    }
  }
}

impl ProtoType {
  pub(crate) fn render_name(&self, current_package: &'static str) -> Cow<'static, str> {
    match self {
      Self::Scalar(scalar) => scalar.to_string().into(),
      Self::Message(path) | Self::Enum(path) => {
        if path.package == current_package {
          path.name.into()
        } else {
          format!("{}.{}", path.package, path.name).into()
        }
      }
    }
  }

  pub(crate) fn register_import(&self, imports: &mut FileImports) {
    match self {
      Self::Scalar { .. } => {}
      Self::Message(path) | Self::Enum(path) => imports.insert_path(path),
    }
  }
}

impl ProtoPath {
  pub(crate) fn render_name(&self, current_package: &'static str) -> Cow<'static, str> {
    if self.package == current_package {
      self.name.into()
    } else {
      format!("{}.{}", self.package, self.name).into()
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtoPath {
  pub name: &'static str,
  pub package: &'static str,
  pub file: &'static str,
}

impl Display for ProtoPath {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let Self { name, package, .. } = self;

    write!(f, "{package}.{name}")
  }
}
