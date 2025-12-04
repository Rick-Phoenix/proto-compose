use std::borrow::Cow;

use crate::*;

pub trait AsProtoType {
  fn proto_type() -> ProtoType;
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProtoType {
  Single(TypeInfo),
  Repeated(TypeInfo),
  Optional(TypeInfo),
  Map { keys: TypeInfo, values: TypeInfo },
}

impl ProtoType {
  pub(crate) fn render(&self, current_package: &'static str) -> Cow<'static, str> {
    let name = self.render_name(current_package);

    match self {
      ProtoType::Single(_) | ProtoType::Map { .. } => name,
      ProtoType::Repeated(_) => format!("repeated {name}").into(),
      ProtoType::Optional(_) => format!("optional {name}").into(),
    }
  }

  pub(crate) fn render_name(&self, current_package: &'static str) -> Cow<'static, str> {
    match self {
      ProtoType::Single(type_info) => type_info.render_name(current_package),
      ProtoType::Repeated(type_info) => type_info.render_name(current_package),
      ProtoType::Optional(type_info) => type_info.render_name(current_package),
      ProtoType::Map { keys, values } => format!(
        "map<{}, {}>",
        keys.render_name(current_package),
        values.render_name(current_package)
      )
      .into(),
    }
  }
}

impl TypeInfo {
  pub(crate) fn render_name(&self, current_package: &'static str) -> Cow<'static, str> {
    if let Some(path) = &self.path  &&
       path.package != current_package {
        format!("{}.{}", path.package, self.name).into()
      } else {
        self.name.into()
      }
  }

  pub(crate) fn register_import(&self, imports: &mut FileImports) {
    if let Some(path) = self.path.as_ref() {
      imports.insert(path)
    }
  }
}

pub(crate) fn invalid_type_output(msg: &'static str) -> TypeInfo {
  TypeInfo {
    name: msg,
    path: None,
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeInfo {
  pub name: &'static str,
  pub path: Option<ProtoPath>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProtoPath {
  pub package: &'static str,
  pub file: &'static str,
}
