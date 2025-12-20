use crate::*;

pub trait ProtoOneof {
  fn proto_schema() -> Oneof;

  fn validate(&self, _parent_messages: &mut Vec<FieldPathElement>) -> Result<(), Violations> {
    Ok(())
  }
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
      options_str = render_normal_options(options);
      options_str.insert(0, '\n');
    }

    format!(
      r"
oneof {name} {{{options_str}
{fields_str}
}}
    ",
    )
  }
}
