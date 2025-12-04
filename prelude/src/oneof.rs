use crate::*;

pub trait ProtoOneof {
  fn fields() -> Vec<ProtoField>;
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Oneof {
  pub name: &'static str,
  pub fields: Vec<ProtoField>,
  pub options: Vec<ProtoOption>,
}

impl Oneof {
  pub(crate) fn render(&self, current_package: &'static str) -> String {
    let Self {
      options,
      fields,
      name,
    } = self;

    let mut fields_str = String::new();

    for (i, field) in fields.iter().enumerate() {
      fields_str.push_str("  ");
      fields_str.push_str(&field.render(current_package));

      if i != fields.len() - 1 {
        fields_str.push('\n');
      }
    }

    let mut options_str = String::new();

    if !options.is_empty() {
      options_str.push('\n');

      for option in options {
        render_option(option, &mut options_str, OptionKind::NormalOption);
        options_str.push('\n');
      }
    }

    format!(
      r###"
oneof {name} {{{options_str}
{fields_str}
}}
    "###,
    )
  }
}
