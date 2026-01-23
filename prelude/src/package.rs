use hashbrown::hash_map::{Entry, HashMap};

use crate::*;

pub struct PackageReference {
  pub name: &'static str,
}

impl PackageReference {
  #[must_use]
  pub const fn new(name: &'static str) -> Self {
    Self { name }
  }

  #[cfg(feature = "inventory")]
  #[must_use]
  pub fn get_package(&self) -> Package {
    collect_package(self.name)
  }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Package {
  pub name: FixedStr,
  pub files: Vec<ProtoFile>,
}

fn insert_message_extern_path(message: &Message, entries: &mut Vec<(String, String)>) {
  let Message {
    name: full_name,
    package,
    rust_path,
    messages,
    enums,
    ..
  } = message;

  let msg_entry = format!(".{package}.{full_name}");

  entries.push((msg_entry, rust_path.clone()));

  for nested_msg in messages {
    insert_message_extern_path(nested_msg, entries);
  }

  for nested_enum in enums {
    insert_enum_extern_path(nested_enum, entries);
  }
}

fn insert_enum_extern_path(enum_: &Enum, entries: &mut Vec<(String, String)>) {
  let Enum {
    name: full_name,
    rust_path,
    package,
    ..
  } = enum_;

  let enum_entry = format!(".{package}.{full_name}");

  entries.push((enum_entry, rust_path.clone()));
}

impl Package {
  #[must_use]
  pub fn extern_paths(&self) -> Vec<(String, String)> {
    let mut entries = Vec::new();

    for file in &self.files {
      for message in &file.messages {
        insert_message_extern_path(message, &mut entries);
      }

      for enum_ in &file.enums {
        insert_enum_extern_path(enum_, &mut entries);
      }
    }

    entries
  }

  #[cfg(feature = "std")]
  pub fn render_files<P>(&self, output_root: P) -> std::io::Result<()>
  where
    P: AsRef<std::path::Path>,
  {
    let output_root = output_root.as_ref();

    std::fs::create_dir_all(output_root)?;

    for file in &self.files {
      let file_path = output_root.join(file.name.as_ref());

      let mut file_buf = std::fs::File::create(file_path)?;

      file.write_into(&mut file_buf)?;
    }

    Ok(())
  }

  #[must_use]
  pub fn new(name: impl Into<FixedStr>) -> Self {
    Self {
      name: name.into(),
      files: Vec::new(),
    }
  }

  #[must_use]
  pub fn get_file(&self, name: &str) -> Option<&ProtoFile> {
    self.files.iter().find(|f| f.name == name)
  }

  #[must_use]
  pub fn with_files(mut self, files: impl IntoIterator<Item = ProtoFile>) -> Self {
    self.files.extend(files.into_iter().map(|mut f| {
      f.package = self.name.clone();
      f
    }));
    self
  }

  #[cfg(feature = "cel")]
  pub fn check_unique_cel_rules(self) -> Result<(), String> {
    for mut message in self.files.into_iter().flat_map(|f| f.messages) {
      let nested_messages = core::mem::take(&mut message.messages);

      for nested in nested_messages {
        check_unique_rules_in_msg(nested)?;
      }

      check_unique_rules_in_msg(message)?;
    }

    Ok(())
  }
}

#[cfg(feature = "cel")]
fn check_unique_rules_in_msg(message: Message) -> Result<(), String> {
  let mut rules: HashMap<FixedStr, CelRule> = HashMap::default();
  let mut duplicates: HashMap<FixedStr, Vec<CelRule>> = HashMap::default();

  for rule in message
    .validators
    .into_iter()
    .flat_map(|v| v.cel_rules)
    .chain(
      message
        .entries
        .into_iter()
        .flat_map(|e| e.cel_rules()),
    )
  {
    let entry = rules.entry(rule.id.clone());

    match entry {
      Entry::Occupied(present) => {
        let present_rule = present.get();

        if *present_rule != rule {
          duplicates
            .entry(rule.id.clone())
            .or_insert_with(|| vec![present_rule.clone()])
            .push(rule);
        }
      }

      Entry::Vacant(vacant) => {
        vacant.insert(rule);
      }
    };
  }

  if !duplicates.is_empty() {
    let mut error = String::new();

    let _ = writeln!(
      error,
      "❌ Found several CEL rules with the same ID in message `{}`:",
      message.name
    );

    for (id, rules) in duplicates {
      writeln!(error, "  Entries for rule ID `{}`:", id.bright_yellow()).unwrap();

      for (i, rule) in rules.iter().enumerate() {
        let rule_str = format!("{rule:#?}");

        let indented_rule = rule_str.replace("\n", "\n  ");

        writeln!(error, "    [{}]: {indented_rule}", i.red()).unwrap();
      }
    }

    return Err(error);
  }

  Ok(())
}
