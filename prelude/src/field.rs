use crate::*;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Field {
  pub name: FixedStr,
  pub tag: i32,
  pub type_: FieldType,
  pub options: Vec<ProtoOption>,
  pub validators: Vec<ValidatorSchema>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ValidatorSchema {
  pub schema: ProtoOption,
  pub cel_rules: Vec<CelRule>,
  pub imports: Vec<FixedStr>,
}

impl Field {
  pub(crate) fn options_with_validators(&self) -> impl Iterator<Item = &options::ProtoOption> {
    self
      .options
      .iter()
      .chain(self.validators.iter().map(|v| &v.schema))
  }

  pub(crate) fn register_import_path(&self, imports: &mut FileImports) {
    for import in self
      .validators
      .iter()
      .flat_map(|v| v.imports.clone())
    {
      imports.insert_internal(import);
    }

    match &self.type_ {
      FieldType::Normal(ty) | FieldType::Repeated(ty) | FieldType::Optional(ty) => {
        ty.register_import(imports)
      }
      FieldType::Map { keys, values } => {
        keys.into_type().register_import(imports);
        values.register_import(imports);
      }
    };
  }
}
