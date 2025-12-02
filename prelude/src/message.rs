use crate::*;

#[derive(Debug, Default, Clone)]
pub struct Message {
  pub name: &'static str,
  pub full_name: &'static str,
  pub package: Arc<str>,
  pub file: Arc<str>,
  pub entries: Vec<MessageEntry>,
  pub messages: Vec<Message>,
  pub enums: Vec<Enum>,
  pub options: Vec<ProtoOption>,
  pub reserved_names: Vec<&'static str>,
  pub reserved_numbers: Vec<Range<i32>>,
}

#[derive(Debug, Clone)]
pub enum MessageEntry {
  Field(ProtoField),
  Oneof(Oneof),
}

impl Message {
  pub fn register_imports(&self, imports: &mut HashSet<Arc<str>>) {
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
  }

  pub fn add_enums<I: IntoIterator<Item = Enum>>(&mut self, enums: I) {
    self.enums = enums.into_iter().collect();
  }
}
