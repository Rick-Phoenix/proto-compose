use crate::*;

pub trait ProtoEnum: TryFrom<i32> + Default {
  fn proto_path() -> ProtoPath;
  fn proto_schema() -> Enum;

  fn full_name() -> &'static str;
}

#[derive(Debug, Default, Clone, PartialEq, Template)]
#[template(path = "enum.proto.j2")]
pub struct Enum {
  pub name: &'static str,
  pub full_name: &'static str,
  pub package: &'static str,
  pub file: &'static str,
  pub variants: Vec<EnumVariant>,
  pub reserved_numbers: Vec<Range<i32>>,
  pub reserved_names: Vec<&'static str>,
  pub options: Vec<ProtoOption>,
  pub rust_path: String,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct EnumVariant {
  pub name: &'static str,
  pub tag: i32,
  pub options: Vec<ProtoOption>,
}

impl Enum {
  pub(crate) fn render_reserved_names(&self) -> Option<String> {
    render_reserved_names(&self.reserved_names)
  }

  pub(crate) fn render_reserved_numbers(&self) -> Option<String> {
    render_reserved_numbers(&self.reserved_numbers)
  }
}
