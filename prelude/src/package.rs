use std::{collections::hash_map::Entry, fs::File, path::Path};

use crate::*;

pub struct PackageGetter {
  pub name: &'static str,
}

impl PackageGetter {
  #[must_use]
  pub const fn new(name: &'static str) -> Self {
    Self { name }
  }

  #[must_use]
  pub fn get_package(&self) -> Package {
    collect_package(self.name)
  }
}

#[derive(Debug)]
pub struct Package {
  pub name: &'static str,
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

  pub fn render_files<P>(&self, output_root: P) -> std::io::Result<()>
  where
    P: AsRef<Path>,
  {
    let output_root = output_root.as_ref();

    std::fs::create_dir_all(output_root)?;

    for file in &self.files {
      let file_path = output_root.join(file.name);

      let mut file_buf = File::create(file_path)?;

      file.write_into(&mut file_buf)?;
    }

    Ok(())
  }

  #[must_use]
  pub const fn new(name: &'static str) -> Self {
    Self {
      name,
      files: Vec::new(),
    }
  }

  pub fn add_files(&mut self, files: impl IntoIterator<Item = ProtoFile>) {
    self.files.extend(files);
  }

  pub fn check_unique_cel_rules(self) -> Result<(), String> {
    let mut rules: FxHashMap<&str, CelRule> = FxHashMap::default();
    let mut duplicates: FxHashMap<&str, Vec<CelRule>> = FxHashMap::default();

    for rule in self
      .files
      .into_iter()
      .flat_map(|f| f.messages.into_iter())
      .flat_map(|message| {
        message.cel_rules.into_iter().chain(
          message
            .entries
            .into_iter()
            .flat_map(|entry| entry.cel_rules()),
        )
      })
    {
      let entry = rules.entry(rule.id);

      match entry {
        Entry::Occupied(present) => {
          let present_rule = present.get();

          if *present_rule != rule {
            duplicates
              .entry(rule.id)
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

      error.push_str("❌ Found one or more CEL rules with same ID but different content:\n");

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
}
