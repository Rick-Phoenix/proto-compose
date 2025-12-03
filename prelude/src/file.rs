use crate::*;

#[derive(Default)]
pub struct ProtoFile {
  pub name: &'static str,
  pub package: &'static str,
  pub imports: Vec<Arc<str>>,
  pub messages: Vec<Message>,
  pub enums: Vec<Enum>,
  // pub services: Vec<ServiceData>,
}

impl ProtoFile {
  pub fn new(name: &'static str, package: &'static str) -> Self {
    Self {
      name,
      package,
      ..Default::default()
    }
  }

  pub fn merge_with(&mut self, other: Self) {
    self.imports.extend(other.imports);
    self.messages.extend(other.messages);
    self.enums.extend(other.enums);
  }

  pub fn add_messages<I: IntoIterator<Item = Message>>(&mut self, messages: I) {
    for message in messages.into_iter() {
      self.messages.push(message);
    }
  }

  pub fn add_enums<I: IntoIterator<Item = Enum>>(&mut self, enums: I) {
    for enum_ in enums.into_iter() {
      self.enums.push(enum_);
    }
  }
}
