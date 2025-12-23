use crate::{validators::CelRule, *};

pub trait ProtoMessage {
  const PACKAGE: &str;

  fn proto_path() -> ProtoPath;
  fn proto_schema() -> Message;

  #[must_use]
  fn cel_rules() -> &'static [&'static CelProgram] {
    &[]
  }

  fn validate(&self) -> Result<(), Violations> {
    Ok(())
  }

  fn name() -> &'static str;

  fn full_name() -> &'static str;

  fn nested_validate(
    &self,
    _field_context: &FieldContext,
    _parent_messages: &mut Vec<FieldPathElement>,
  ) -> Result<(), Violations> {
    Ok(())
  }
}

impl<T> ProtoMessage for Box<T>
where
  T: ProtoMessage,
{
  const PACKAGE: &str = T::PACKAGE;

  fn full_name() -> &'static str {
    T::full_name()
  }

  fn name() -> &'static str {
    T::name()
  }

  fn proto_path() -> ProtoPath {
    T::proto_path()
  }

  fn proto_schema() -> Message {
    T::proto_schema()
  }
}

#[derive(Debug, Default, Clone, PartialEq, Template)]
#[template(path = "message.proto.j2")]
pub struct Message {
  pub short_name: &'static str,
  pub name: &'static str,
  pub package: &'static str,
  pub file: &'static str,
  pub entries: Vec<MessageEntry>,
  pub messages: Vec<Self>,
  pub enums: Vec<Enum>,
  pub options: Vec<ProtoOption>,
  pub reserved_names: Vec<&'static str>,
  pub reserved_numbers: Vec<Range<i32>>,
  pub cel_rules: Vec<&'static CelRule>,
  pub rust_path: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageEntry {
  Field(ProtoField),
  Oneof(Oneof),
}

impl MessageEntry {
  pub(crate) fn cel_rules(&self) -> impl Iterator<Item = &'static CelRule> {
    let fields_slice = match self {
      Self::Field(f) => std::slice::from_ref(f),
      Self::Oneof(o) => o.fields.as_slice(),
    };

    fields_slice
      .iter()
      .flat_map(|f| f.validator.iter())
      .flat_map(|v| v.cel_rules.iter().copied())
  }
}

impl Message {
  pub(crate) fn render_reserved_names(&self) -> Option<String> {
    render_reserved_names(&self.reserved_names)
  }

  pub(crate) fn render_reserved_numbers(&self) -> Option<String> {
    render_reserved_numbers(&self.reserved_numbers)
  }

  pub(crate) fn register_imports(&self, imports: &mut FileImports) {
    if !self.cel_rules.is_empty() {
      imports.set.insert("buf/validate/validate.proto");
    }

    for entry in &self.entries {
      match entry {
        MessageEntry::Field(field) => field.register_type_import_path(imports),
        MessageEntry::Oneof(oneof) => {
          for field in &oneof.fields {
            field.register_type_import_path(imports)
          }
        }
      }
    }

    for nested_msg in &self.messages {
      nested_msg.register_imports(imports);
    }
  }

  pub fn add_enums<I: IntoIterator<Item = Enum>>(&mut self, enums: I) {
    self.enums = enums.into_iter().collect();
  }
}
