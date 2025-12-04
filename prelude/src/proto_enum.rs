use crate::*;

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
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct EnumVariant {
  pub name: &'static str,
  pub tag: i32,
  pub options: Vec<ProtoOption>,
}
