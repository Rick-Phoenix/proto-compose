use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct ProtoField {
  pub name: String,
  pub tag: i32,
  pub type_: ProtoType,
  pub options: Vec<ProtoOption>,
  pub validator: Option<ProtoOption>,
}

impl ProtoField {
  pub(crate) fn render(&self, current_package: &'static str) -> String {
    let mut field = format!(
      "{} {} = {}",
      self.type_.render(current_package),
      self.name,
      self.tag
    );

    let mut options = self.options.clone();

    if let Some(validator) = &self.validator {
      options.push(validator.clone());
    }

    if !options.is_empty() {
      field.push_str(" [\n");

      for (i, option) in options.iter().enumerate() {
        for line in option.render_as_field_option().lines() {
          field.push_str("  ");
          field.push_str(line);

          field.push('\n');
        }

        if i != options.len() - 1 {
          field.push_str(",\n");
        }
      }

      field.push_str("\n]");
    }

    field.push(';');

    field
  }

  pub(crate) fn register_type_import_path(&self, imports: &mut FileImports) {
    match &self.type_ {
      ProtoType::Single(ty) => ty.register_import(imports),
      ProtoType::Repeated(ty) => ty.register_import(imports),
      ProtoType::Optional(ty) => ty.register_import(imports),
      ProtoType::Map { keys, values } => {
        keys.register_import(imports);
        values.register_import(imports);
      }
    };
  }
}
