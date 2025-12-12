use std::collections::hash_map::Entry;

use crate::*;

pub struct Package {
  pub files: Vec<ProtoFile>,
}

impl Package {
  pub fn check_cel_rules(&self) -> Result<(), String> {
    let mut rules: HashMap<&str, &CelRule> = HashMap::new();

    for rule in self
      .files
      .iter()
      .flat_map(|f| f.messages.iter())
      .flat_map(|m| m.cel_rules.iter())
    {
      let entry = rules.entry(&rule.id);

      match entry {
        Entry::Occupied(present) => {
          let present_rule = present.get();
          if *present_rule != rule {
            return Err(format!(
              "Found multiple CEL rules with ID `{}`.\nRule 1: {present_rule:#?}\nRule 2: {rule:#?}",
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
