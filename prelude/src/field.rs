use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
  pub name: &'static str,
  pub tag: i32,
  pub type_: FieldType,
  pub options: Vec<ProtoOption>,
  pub validator: Option<FieldValidator>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldValidator {
  pub schema: ProtoOption,
  pub cel_rules: Vec<CelRule>,
}

impl Field {
  pub(crate) fn options_with_validators(&self) -> impl Iterator<Item = &options::ProtoOption> {
    self
      .options
      .iter()
      .chain(self.validator.iter().map(|v| &v.schema))
  }

  pub(crate) fn register_import_path(&self, imports: &mut FileImports) {
    if self.validator.is_some() {
      imports.insert_validate_proto();
    }

    match &self.type_ {
      FieldType::Normal(ty) | FieldType::Repeated(ty) | FieldType::Optional(ty) => {
        ty.register_import(imports)
      }
      FieldType::Map { keys, values } => {
        keys.register_import(imports);
        values.register_import(imports);
      }
    };
  }
}
