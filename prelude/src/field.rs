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
    let Self {
      tag,
      type_,
      name,
      options,
      validator,
    } = self;

    let mut field_str = format!("{} {} = {}", type_.render(current_package), name, tag);

    let options_iter = options.iter().chain(validator.iter()).enumerate();
    let options_len = if validator.is_some() {
      options.len() + 1
    } else {
      options.len()
    };

    if options_len != 0 {
      field_str.push_str(" [\n");

      for (i, option) in options_iter {
        render_option(option, &mut field_str, OptionKind::FieldOption);

        if i != options_len - 1 {
          field_str.push_str(",\n");
        }
      }

      field_str.push_str("\n]");
    }

    field_str.push(';');

    field_str
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
