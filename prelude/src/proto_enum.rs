use crate::*;

pub trait ProtoEnum: TryFrom<i32> {
  fn proto_path() -> ProtoPath;
  fn proto_schema() -> Enum;
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
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct EnumVariant {
  pub name: &'static str,
  pub tag: i32,
  pub options: Vec<ProtoOption>,
}

impl EnumVariant {
  pub(crate) fn render(&self) -> String {
    let Self { tag, name, options } = self;

    let mut variant_str = format!("{name} = {tag}");

    if !options.is_empty() {
      render_field_options(options.iter().enumerate(), options.len(), &mut variant_str);
    }

    variant_str.push(';');

    variant_str
  }
}

impl Enum {
  pub(crate) fn render_reserved_names(&self) -> Option<String> {
    render_reserved_names(&self.reserved_names)
  }

  pub(crate) fn render_reserved_numbers(&self) -> Option<String> {
    render_reserved_numbers(&self.reserved_numbers)
  }

  pub(crate) fn render_options(&self) -> Option<String> {
    if self.options.is_empty() {
      return None;
    }

    Some(render_normal_options(&self.options))
  }
}
