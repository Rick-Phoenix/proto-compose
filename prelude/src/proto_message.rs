use crate::{validators::CelRule, *};

pub trait ProxiedMessage: From<Self::Proxy> + Into<Self::Proxy> {
  type Proxy: MessageProxy<Message = Self> + From<Self> + Into<Self>;

  #[inline]
  fn into_proxy(self) -> Self::Proxy {
    self.into()
  }
}

pub trait MessageProxy: From<Self::Message> + Into<Self::Message> {
  type Message: ProtoMessage + ValidatedMessage + From<Self> + Into<Self>;

  #[inline]
  fn into_message(self) -> Self::Message {
    self.into()
  }

  #[inline]
  fn into_validated_message(self) -> Result<Self::Message, ValidationErrors> {
    let msg = self.into_message();

    match msg.validate() {
      Ok(()) => Ok(msg),
      Err(e) => Err(e),
    }
  }

  #[inline]
  fn from_validated_message(msg: Self::Message) -> Result<Self, ValidationErrors> {
    match msg.validate() {
      Ok(()) => Ok(Self::from(msg)),
      Err(e) => Err(e),
    }
  }
}

pub trait MessagePath {
  fn proto_path() -> ProtoPath;
}

pub trait ProtoMessage: Default + MessagePath {
  const PACKAGE: &str;
  const SHORT_NAME: &str;

  fn proto_schema() -> Message;

  fn proto_name() -> &'static str;
  fn full_name() -> &'static str;
  fn type_url() -> &'static str;
}

#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Template))]
#[cfg_attr(feature = "std", template(path = "message.proto.j2"))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Message {
  pub short_name: FixedStr,
  pub name: FixedStr,
  pub package: FixedStr,
  pub file: FixedStr,
  pub entries: Vec<MessageEntry>,
  pub messages: Vec<Self>,
  pub enums: Vec<Enum>,
  pub options: Vec<ProtoOption>,
  pub reserved_names: Vec<FixedStr>,
  pub reserved_numbers: Vec<Range<i32>>,
  pub validators: Vec<ValidatorSchema>,
  pub rust_path: FixedStr,
}

impl Message {
  pub(crate) fn options_with_validators(&self) -> Vec<ProtoOption> {
    self
      .options
      .clone()
      .into_iter()
      .chain(self.validators.iter().map(|v| v.schema.clone()))
      .collect()
  }

  pub fn fields(&self) -> impl Iterator<Item = &Field> {
    self.entries.iter().flat_map(|entry| {
      let (field_opt, oneof_vec) = match entry {
        MessageEntry::Field(f) => (Some(f), None),
        MessageEntry::Oneof(oneof) => (None, Some(&oneof.fields)),
      };

      field_opt
        .into_iter()
        .chain(oneof_vec.into_iter().flatten())
    })
  }

  #[must_use]
  pub fn with_nested_messages(mut self, messages: impl IntoIterator<Item = Self>) -> Self {
    self.messages.extend(messages);
    self
  }

  #[must_use]
  pub fn with_nested_enums(mut self, enums: impl IntoIterator<Item = Enum>) -> Self {
    self.enums.extend(enums);
    self
  }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MessageEntry {
  Field(Field),
  Oneof(Oneof),
}

impl MessageEntry {
  pub(crate) fn cel_rules(self) -> impl Iterator<Item = CelRule> {
    let (field_opt, oneof_vec) = match self {
      Self::Field(f) => (Some(f), None),
      Self::Oneof(oneof) => (None, Some(oneof.fields)),
    };

    field_opt
      .into_iter()
      .chain(oneof_vec.into_iter().flatten())
      .flat_map(|f| f.validators)
      .flat_map(|v| v.cel_rules)
  }

  /// Returns `true` if the message entry is [`Field`].
  ///
  /// [`Field`]: MessageEntry::Field
  #[must_use]
  pub const fn is_field(&self) -> bool {
    matches!(self, Self::Field(..))
  }

  /// Returns `true` if the message entry is [`Oneof`].
  ///
  /// [`Oneof`]: MessageEntry::Oneof
  #[must_use]
  pub const fn is_oneof(&self) -> bool {
    matches!(self, Self::Oneof { .. })
  }

  #[must_use]
  pub const fn as_field(&self) -> Option<&Field> {
    if let Self::Field(v) = self {
      Some(v)
    } else {
      None
    }
  }

  #[must_use]
  pub const fn as_oneof(&self) -> Option<&Oneof> {
    if let Self::Oneof(v) = self {
      Some(v)
    } else {
      None
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

  pub(crate) fn register_imports(&self, imports: &mut FileImports) {
    for import in self
      .validators
      .iter()
      .flat_map(|v| v.imports.clone())
    {
      imports.insert_internal(import);
    }

    for entry in &self.entries {
      match entry {
        MessageEntry::Field(field) => field.register_import_path(imports),
        MessageEntry::Oneof(oneof) => {
          for field in &oneof.fields {
            field.register_import_path(imports)
          }
        }
      }
    }

    for nested_msg in &self.messages {
      nested_msg.register_imports(imports);
    }
  }
}
