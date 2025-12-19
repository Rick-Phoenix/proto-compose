use std::collections::hash_map::Entry;

use crate::*;

pub struct Package {
  pub name: &'static str,
  pub files: Vec<ProtoFile>,
}

impl Package {
  pub fn new(name: &'static str) -> Self {
    Self {
      name,
      files: Vec::new(),
    }
  }

  pub fn add_files(&mut self, files: impl IntoIterator<Item = ProtoFile>) {
    self.files.extend(files);
  }

  #[cfg(feature = "testing")]
  pub fn check_unique_cel_rules(&self) -> Result<(), String> {
    let mut rules: HashMap<&str, &CelRule> = HashMap::new();
    let mut duplicates: HashMap<&str, Vec<&CelRule>> = HashMap::new();

    for rule in self
      .files
      .iter()
      .flat_map(|f| f.messages.iter())
      .flat_map(|message| {
        message.cel_rules.iter().copied().chain(
          message
            .entries
            .iter()
            .flat_map(|entry| entry.cel_rules()),
        )
      })
    {
      let entry = rules.entry(&rule.id);

      match entry {
        Entry::Occupied(present) => {
          let present_rule = present.get();

          if *present_rule != rule {
            duplicates
              .entry(&rule.id)
              .or_insert_with(|| vec![present_rule])
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

      error.push_str("Found one or more CEL rules with same ID but different content:\n");

      for (id, rules) in duplicates {
        writeln!(error, "  Entries for rule ID `{id}`:").unwrap();

        for (i, rule) in rules.iter().enumerate() {
          let rule_str = format!("{rule:#?}");

          let indented_rule = rule_str.replace("\n", "\n  ");

          writeln!(error, "    [{i}]: {indented_rule}").unwrap();
        }
      }

      return Err(error);
    }

    Ok(())
  }
}
