use std::fmt::Display;

use crate::*;

#[derive(Debug, PartialEq, Template)]
#[template(path = "file.proto.j2")]
pub struct ProtoFile {
  pub name: &'static str,
  pub package: &'static str,
  pub imports: FileImports,
  pub messages: Vec<Message>,
  pub enums: Vec<Enum>,
  pub options: Vec<ProtoOption>,
  pub edition: Edition,
  pub services: Vec<Service>,
  pub extensions: Vec<Extension>,
}

#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub enum Edition {
  Proto2,
  #[default]
  Proto3,
  E2023,
}

impl Display for Edition {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Edition::Proto2 => write!(f, "syntax = \"proto2\""),
      Edition::Proto3 => write!(f, "syntax = \"proto3\""),
      Edition::E2023 => write!(f, "edition = \"2023\""),
    }
  }
}

#[derive(PartialEq, Debug)]
pub struct FileImports {
  pub set: HashSet<&'static str>,
  pub file: &'static str,
}

impl FileImports {
  pub fn extend(&mut self, other: FileImports) {
    self.set.extend(other.set);
  }

  pub fn new(file: &'static str) -> Self {
    Self {
      file,
      set: HashSet::new(),
    }
  }

  pub fn insert_path(&mut self, path: &ProtoPath) {
    if path.file != self.file {
      self.set.insert(path.file);
    }
  }

  pub fn as_sorted_vec(&self) -> Vec<&'static str> {
    let mut imports: Vec<&'static str> = self.set.iter().cloned().collect();

    imports.sort();

    imports
  }
}

impl ProtoFile {
  pub fn new(name: &'static str, package: &'static str) -> Self {
    Self {
      name,
      package,
      imports: FileImports::new(name),
      messages: Default::default(),
      enums: Default::default(),
      options: Default::default(),
      edition: Default::default(),
      services: Default::default(),
      extensions: Default::default(),
    }
  }

  pub fn edition(&mut self, edition: Edition) {
    self.edition = edition;
  }

  pub fn merge_with(&mut self, other: Self) {
    self.imports.extend(other.imports);
    self.messages.extend(other.messages);
    self.enums.extend(other.enums);
  }

  pub fn add_messages<I: IntoIterator<Item = Message>>(&mut self, messages: I) {
    for message in messages.into_iter() {
      message.register_imports(&mut self.imports);

      self.messages.push(message);
    }
  }

  pub fn add_enums<I: IntoIterator<Item = Enum>>(&mut self, enums: I) {
    for enum_ in enums.into_iter() {
      self.enums.push(enum_);
    }
  }

  pub fn add_services<I: IntoIterator<Item = Service>>(&mut self, services: I) {
    for service in services.into_iter() {
      self.services.push(service);
    }
  }

  pub fn add_extensions<I: IntoIterator<Item = Extension>>(&mut self, extensions: I) {
    self.imports.set.insert("google/protobuf/descriptor.proto");

    for ext in extensions.into_iter() {
      self.extensions.push(ext);
    }
  }
}

impl ProtoFile {
  pub(crate) fn render_options(&self) -> Option<String> {
    if self.options.is_empty() {
      return None;
    }

    let mut options_str = String::new();

    for option in &self.options {
      render_option(option, &mut options_str, OptionKind::NormalOption);
      options_str.push('\n');
    }

    Some(options_str)
  }
}

#[derive(Debug, PartialEq)]
pub struct Extension {
  pub target: &'static str,
  pub fields: Vec<ProtoField>,
}
