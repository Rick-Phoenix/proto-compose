use std::collections::hash_map::Entry;

use crate::*;

pub struct Package {
  pub files: Vec<ProtoFile>,
}

impl Package {
  #[cfg(feature = "testing")]
  pub fn check_unique_cel_rules(&self) -> Result<(), String> {
    let mut rules: HashMap<&str, &CelRule> = HashMap::new();

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
            return Err(format!(
              "Found multiple different CEL rules with ID `{}`.\nRule 1: {present_rule:#?}\nRule 2: {rule:#?}",
              rule.id
            ));
          }
        }

        Entry::Vacant(vacant) => {
          vacant.insert(rule);
        }
      };
    }

    Ok(())
  }
}
