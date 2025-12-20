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

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Edition {
  Proto2,
  #[default]
  Proto3,
  E2023,
}

impl Display for Edition {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Proto2 => write!(f, "syntax = \"proto2\""),
      Self::Proto3 => write!(f, "syntax = \"proto3\""),
      Self::E2023 => write!(f, "edition = \"2023\""),
    }
  }
}

#[derive(PartialEq, Eq, Debug)]
pub struct FileImports {
  pub set: HashSet<&'static str>,
  pub file: &'static str,
}

impl FileImports {
  pub fn extend(&mut self, other: Self) {
    self.set.extend(other.set);
  }

  #[must_use]
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

  #[must_use]
  pub fn as_sorted_vec(&self) -> Vec<&'static str> {
    let mut imports: Vec<&'static str> = self.set.iter().copied().collect();

    imports.sort_unstable();

    imports
  }
}

impl ProtoFile {
  #[must_use]
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

  pub const fn edition(&mut self, edition: Edition) {
    self.edition = edition;
  }

  pub fn merge_with(&mut self, other: Self) {
    if self.name != other.name {
      panic!(
        "Cannot merge file `{}` with file `{}` as they have different names",
        self.name, other.name
      );
    }

    if self.package != other.package {
      panic!(
        "Cannot merge file `{}` with file `{}` as they belong to different packages",
        self.name, other.name
      );
    }

    self.imports.extend(other.imports);
    self.messages.extend(other.messages);
    self.enums.extend(other.enums);
    self.services.extend(other.services);
    self.extensions.extend(other.extensions);
    self.options.extend(other.options);
  }

  pub fn add_messages<I: IntoIterator<Item = Message>>(&mut self, messages: I) {
    for message in messages {
      message.register_imports(&mut self.imports);

      self.messages.push(message);
    }
  }

  pub fn add_enums<I: IntoIterator<Item = Enum>>(&mut self, enums: I) {
    for enum_ in enums {
      self.enums.push(enum_);
    }
  }

  pub fn add_services<I: IntoIterator<Item = Service>>(&mut self, services: I) {
    for service in services {
      self.services.push(service);
    }
  }

  pub fn add_extensions<I: IntoIterator<Item = Extension>>(&mut self, extensions: I) {
    self
      .imports
      .set
      .insert("google/protobuf/descriptor.proto");

    for ext in extensions {
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
