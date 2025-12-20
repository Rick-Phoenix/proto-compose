use crate::{validators::CelRule, *};

pub trait ProtoMessage {
  fn proto_path() -> ProtoPath;
  fn proto_schema() -> Message;

  fn validate(&self) -> Result<(), Violations> {
    Ok(())
  }

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
  pub name: &'static str,
  pub full_name: &'static str,
  pub package: &'static str,
  pub file: &'static str,
  pub entries: Vec<MessageEntry>,
  pub messages: Vec<Self>,
  pub enums: Vec<Enum>,
  pub options: Vec<ProtoOption>,
  pub reserved_names: Vec<&'static str>,
  pub reserved_numbers: Vec<Range<i32>>,
  pub cel_rules: Vec<&'static CelRule>,
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

  pub(crate) fn render(&self, current_package: &'static str) -> String {
    match self {
      Self::Field(proto_field) => proto_field.render(current_package),
      Self::Oneof(oneof) => oneof.render(current_package),
    }
  }
}

impl Message {
  pub(crate) fn render_reserved_names(&self) -> Option<String> {
    render_reserved_names(&self.reserved_names)
  }

  pub(crate) fn render_reserved_numbers(&self) -> Option<String> {
    render_reserved_numbers(&self.reserved_numbers)
  }

  pub(crate) fn render_options(&self) -> Option<String> {
    if self.cel_rules.is_empty() && self.options.is_empty() {
      return None;
    }

    let cel_rules_options: Vec<ProtoOption> = self
      .cel_rules
      .iter()
      .map(|&rule| rule.clone().into())
      .collect();

    let options = self
      .options
      .iter()
      .chain(cel_rules_options.iter());

    Some(render_normal_options(options))
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
