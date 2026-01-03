use fxhash::FxBuildHasher;
use ordermap::OrderMap;

use crate::*;

pub struct RegistryPath {
  pub file: &'static str,
  pub package: &'static str,
  pub extern_path: &'static str,
}

type Map<K, V> = OrderMap<K, V, FxBuildHasher>;

fn process_msg(
  msg_name: &str,
  messages: &mut Map<&'static str, Message>,
  enums: &mut Map<&'static str, Enum>,
  parent_messages_map: &mut Map<&'static str, NestedItems>,
) -> Message {
  let mut msg = messages
    .swap_remove(msg_name)
    .unwrap_or_else(|| panic!("Could not find message {msg_name}"));

  let Some(children) = parent_messages_map.swap_remove(msg_name) else {
    return msg;
  };

  for child in children.messages {
    let child_data = process_msg(child, messages, enums, parent_messages_map);

    msg.messages.push(child_data);
  }

  for enum_ in children.enums {
    let enum_data = enums
      .swap_remove(enum_)
      .unwrap_or_else(|| panic!("Could not find enum {enum_}"));

    msg.enums.push(enum_data);
  }

  msg
}

#[derive(Debug, Default)]
struct NestedItems {
  pub enums: Vec<&'static str>,
  pub messages: Vec<&'static str>,
}

#[must_use]
pub fn collect_package(package: &'static str) -> Package {
  let mut messages: Map<&'static str, Message> = Map::default();
  let mut enums: Map<&'static str, Enum> = Map::default();
  let mut parent_messages_map: Map<&'static str, NestedItems> = Map::default();
  let mut root_messages: Vec<&'static str> = Vec::new();
  let mut files: Map<&'static str, ProtoFile> = Map::default();

  for file_entry in inventory::iter::<RegistryFile>().filter(|f| f.package == package) {
    let file: ProtoFile = file_entry.into();

    match files.entry(file.name) {
      ordermap::map::Entry::Occupied(mut occupied) => {
        occupied.get_mut().merge_with(file);
      }
      ordermap::map::Entry::Vacant(vacant) => {
        vacant.insert(file);
      }
    };
  }

  for msg_entry in inventory::iter::<RegistryMessage>().filter(|rm| rm.package == package) {
    let msg = (msg_entry.message)();

    if let Some(parent_getter) = msg_entry.parent_message {
      let parent = parent_getter();

      parent_messages_map
        .entry(parent)
        .or_default()
        .messages
        .push(msg.name);
    } else {
      root_messages.push(msg.name);
    }

    messages.insert(msg.name, msg);
  }

  for enum_entry in inventory::iter::<RegistryEnum>().filter(|rm| rm.package == package) {
    let enum_ = (enum_entry.enum_)();

    if let Some(parent_getter) = enum_entry.parent_message {
      let parent = parent_getter();

      parent_messages_map
        .entry(parent)
        .or_default()
        .enums
        .push(enum_.name);

      enums.insert(enum_.name, enum_);
    } else {
      files
        .get_mut(enum_.file)
        .unwrap_or_else(|| panic!("Could not find the data for file {}", enum_.file))
        .enums
        .push(enum_);
    }
  }

  for root_message_name in root_messages {
    let msg = process_msg(
      root_message_name,
      &mut messages,
      &mut enums,
      &mut parent_messages_map,
    );

    files
      .get_mut(msg.file)
      .unwrap_or_else(|| panic!("Could not find the data for file {}", msg.file))
      .add_messages([msg]);
  }

  for service_entry in inventory::iter::<RegistryService>().filter(|rs| rs.package == package) {
    let service = (service_entry.service)();

    files
      .get_mut(service.file)
      .unwrap_or_else(|| panic!("Could not find the data for file {}", service.file))
      .add_services([service]);
  }

  let files: Vec<ProtoFile> = files
    .into_values()
    .map(|mut file| {
      file
        .extensions
        .sort_unstable_by_key(|e| e.target.as_str());

      file.messages.sort_unstable_by_key(|m| m.name);

      for msg in file.messages.iter_mut() {
        msg.messages.sort_unstable_by_key(|m| m.name);
        msg.enums.sort_unstable_by_key(|e| e.short_name);
      }

      file.enums.sort_unstable_by_key(|e| e.short_name);
      file.services.sort_unstable_by_key(|s| s.name);

      file
    })
    .collect();

  Package {
    name: package,
    files,
  }
}

pub struct RegistryMessage {
  pub package: &'static str,
  pub parent_message: Option<fn() -> &'static str>,
  pub message: fn() -> Message,
}

pub struct RegistryEnum {
  pub package: &'static str,
  pub parent_message: Option<fn() -> &'static str>,
  pub enum_: fn() -> Enum,
}

pub struct RegistryService {
  pub package: &'static str,
  pub service: fn() -> Service,
}

pub struct RegistryFile {
  pub file: &'static str,
  pub package: &'static str,
  pub options: fn() -> Vec<ProtoOption>,
  pub imports: fn() -> Vec<&'static str>,
  pub extensions: fn() -> Vec<Extension>,
}

#[allow(clippy::from_over_into)]
impl Into<ProtoFile> for &RegistryFile {
  fn into(self) -> ProtoFile {
    let mut file = ProtoFile::new(self.file, self.package);

    file.imports.extend((self.imports)());
    file.add_extensions((self.extensions)());
    file.options = (self.options)();

    file
  }
}

inventory::collect!(RegistryMessage);
inventory::collect!(RegistryEnum);
inventory::collect!(RegistryService);
inventory::collect!(RegistryFile);
