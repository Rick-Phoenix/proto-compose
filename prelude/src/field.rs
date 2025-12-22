use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct ProtoField {
  pub name: String,
  pub tag: i32,
  pub type_: ProtoFieldInfo,
  pub options: Vec<ProtoOption>,
  pub validator: Option<FieldValidator>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldValidator {
  pub schema: ProtoOption,
  pub cel_rules: Vec<&'static CelRule>,
}

impl ProtoField {
  pub(crate) fn options_with_validators(&self) -> impl Iterator<Item = &options::ProtoOption> {
    self
      .options
      .iter()
      .chain(self.validator.iter().map(|v| &v.schema))
  }

  pub(crate) fn register_type_import_path(&self, imports: &mut FileImports) {
    if self.validator.is_some() {
      imports.set.insert("buf/validate/validate.proto");
    }

    match &self.type_ {
      ProtoFieldInfo::Single(ty) | ProtoFieldInfo::Repeated(ty) | ProtoFieldInfo::Optional(ty) => {
        ty.register_import(imports)
      }
      ProtoFieldInfo::Map { keys, values } => {
        keys.register_import(imports);
        values.register_import(imports);
      }
    };
  }
}
