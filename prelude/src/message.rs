use crate::{validators::CelRule, *};

pub trait IntoMessage: From<Self::Message> + Into<Self::Message> {
  type Message: ::prost::Message + ProtoMessage + ValidatedMessage + From<Self>;

  fn into_message(self) -> Self::Message {
    self.into()
  }

  fn from_message(msg: Self::Message) -> Self {
    msg.into()
  }

  fn into_validated_message(self) -> Result<Self::Message, Violations> {
    let msg = self.into_message();

    match msg.validate() {
      Ok(()) => Ok(msg),
      Err(e) => Err(e),
    }
  }

  fn from_validated_message(msg: Self::Message) -> Result<Self, Violations> {
    match msg.validate() {
      Ok(()) => Ok(Self::from_message(msg)),
      Err(e) => Err(e),
    }
  }
}

impl<T: IntoMessage> ProtoMessage for T {
  const PACKAGE: &str = T::Message::PACKAGE;
  const SHORT_NAME: &str = T::Message::SHORT_NAME;

  fn proto_path() -> ProtoPath {
    T::Message::proto_path()
  }

  fn proto_schema() -> Message {
    T::Message::proto_schema()
  }

  fn name() -> &'static str {
    T::Message::name()
  }

  fn full_name() -> &'static str {
    T::Message::full_name()
  }

  fn type_url() -> &'static str {
    T::Message::type_url()
  }
}

pub trait ProtoMessage {
  const PACKAGE: &str;
  const SHORT_NAME: &str;

  fn proto_path() -> ProtoPath;
  fn proto_schema() -> Message;

  fn name() -> &'static str;
  fn full_name() -> &'static str;
  fn type_url() -> &'static str;
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
  pub cel_rules: Vec<CelRule>,
  // Not a static str because we compose this
  // by default with module_path!() + ident
  pub rust_path: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageEntry {
  Field(ProtoField),
  Oneof { oneof: Oneof, required: bool },
}

impl MessageEntry {
  pub(crate) fn cel_rules(self) -> impl Iterator<Item = CelRule> {
    let (field_opt, oneof_vec) = match self {
      Self::Field(f) => (Some(f), None),
      Self::Oneof { oneof, .. } => (None, Some(oneof.fields)),
    };

    field_opt
      .into_iter()
      .chain(oneof_vec.into_iter().flatten())
      .filter_map(|f| f.validator)
      .flat_map(|v| v.cel_rules)
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
      imports.insert_validate_proto();
    }

    for entry in &self.entries {
      match entry {
        MessageEntry::Field(field) => field.register_type_import_path(imports),
        MessageEntry::Oneof {
          oneof, required, ..
        } => {
          if *required {
            imports.insert_validate_proto();
          }

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
