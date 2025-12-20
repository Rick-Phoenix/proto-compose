use std::borrow::Cow;

use crate::*;

pub trait AsProtoType {
  fn proto_type() -> ProtoType;
}

pub trait AsProtoField {
  fn as_proto_field() -> ProtoFieldInfo;
}

impl<T: AsProtoType> AsProtoField for T {
  fn as_proto_field() -> ProtoFieldInfo {
    ProtoFieldInfo::Single(T::proto_type())
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtoFieldInfo {
  Single(ProtoType),
  Map { keys: ProtoType, values: ProtoType },
  Repeated(ProtoType),
  Optional(ProtoType),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtoType {
  Primitive { name: &'static str },
  Message(ProtoPath),
  Enum(ProtoPath),
}

impl ProtoType {
  /// Returns `true` if the proto type2 is [`Message`].
  ///
  /// [`Message`]: ProtoType::Message
  #[must_use]
  pub const fn is_message(&self) -> bool {
    matches!(self, Self::Message { .. })
  }
}

impl ProtoFieldInfo {
  pub(crate) fn render(&self, current_package: &'static str) -> Cow<'static, str> {
    let name = self.render_name(current_package);

    match self {
      Self::Single(_) | Self::Map { .. } => name,
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
      Self::Single(type_info) | Self::Repeated(type_info) | Self::Optional(type_info) => {
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
      Self::Primitive { name } => (*name).into(),
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
      Self::Primitive { .. } => {}
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
