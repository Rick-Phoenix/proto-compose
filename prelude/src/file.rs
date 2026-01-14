use crate::*;
use hashbrown::HashSet;

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

pub struct FileReference {
  pub name: &'static str,
  pub package: &'static str,
  pub extern_path: &'static str,
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Edition {
  Proto2,
  #[default]
  Proto3,
  E2023,
}

impl Display for Edition {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::Proto2 => write!(f, "syntax = \"proto2\""),
      Self::Proto3 => write!(f, "syntax = \"proto3\""),
      Self::E2023 => write!(f, "edition = \"2023\""),
    }
  }
}

#[doc(hidden)]
#[derive(PartialEq, Eq, Debug)]
pub struct FileImports {
  pub set: HashSet<&'static str>,
  pub file: &'static str,
  pub added_validate_proto: bool,
}

impl Extend<&'static str> for FileImports {
  fn extend<T: IntoIterator<Item = &'static str>>(&mut self, iter: T) {
    self.set.extend(iter);
  }
}

impl IntoIterator for FileImports {
  type Item = &'static str;
  type IntoIter = hashbrown::hash_set::IntoIter<&'static str>;

  fn into_iter(self) -> Self::IntoIter {
    self.set.into_iter()
  }
}

impl FileImports {
  #[must_use]
  pub fn new(file: &'static str) -> Self {
    Self {
      file,
      set: HashSet::default(),
      added_validate_proto: false,
    }
  }

  pub fn insert_validate_proto(&mut self) {
    if !self.added_validate_proto {
      self.set.insert("buf/validate/validate.proto");
      self.added_validate_proto = true;
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

  #[track_caller]
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
    for mut message in messages {
      message.register_imports(&mut self.imports);
      message.file = self.name;

      self.messages.push(message);
    }
  }

  pub fn add_enums<I: IntoIterator<Item = Enum>>(&mut self, enums: I) {
    for mut enum_ in enums {
      enum_.file = self.name;

      self.enums.push(enum_);
    }
  }

  pub fn add_services<I: IntoIterator<Item = Service>>(&mut self, services: I) {
    for service in services {
      for (request, response) in service
        .handlers
        .iter()
        .map(|h| (&h.request, &h.response))
      {
        self.imports.insert_path(request);
        self.imports.insert_path(response);
      }

      if service.file != self.name {
        self.imports.set.insert(service.file);
      }

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
