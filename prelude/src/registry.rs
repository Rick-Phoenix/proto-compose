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

  for entry in inventory::iter::<RegistryMessage>().filter(|rm| rm.package == package) {
    let msg = (entry.message)();

    if let Some(parent_getter) = entry.parent_message {
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
        .push(enum_.full_name);

      enums.insert(enum_.full_name, enum_);
    } else {
      files
        .entry(enum_.file)
        .or_insert_with(|| ProtoFile::new(enum_.file, package))
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
      .entry(msg.file)
      .or_insert_with(|| ProtoFile::new(msg.file, package))
      .add_messages([msg]);
  }

  for service in inventory::iter::<RegistryService>().filter(|rs| rs.package == package) {
    let service_data = (service.service)();

    files
      .entry(service.file)
      .or_insert_with(|| ProtoFile::new(service.file, package))
      .add_services([service_data]);
  }

  for extension in inventory::iter::<RegistryExtension>().filter(|re| re.package == package) {
    let ext_data = (extension.extension)();

    files
      .entry(extension.file)
      .or_insert_with(|| ProtoFile::new(extension.file, package))
      .add_extensions([ext_data]);
  }

  for option in inventory::iter::<RegistryFileOptions>().filter(|rfo| rfo.package == package) {
    let options = (option.options)();

    files
      .entry(option.file)
      .or_insert_with(|| ProtoFile::new(option.file, package))
      .options
      .extend(options);
  }

  Package {
    name: package,
    files: files.into_values().collect(),
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
  pub file: &'static str,
  pub service: fn() -> Service,
}

pub struct RegistryExtension {
  pub package: &'static str,
  pub file: &'static str,
  pub extension: fn() -> Extension,
}

pub struct RegistryFileOptions {
  pub file: &'static str,
  pub package: &'static str,
  pub options: fn() -> Vec<ProtoOption>,
}

inventory::collect!(RegistryMessage);
inventory::collect!(RegistryEnum);
inventory::collect!(RegistryService);
inventory::collect!(RegistryExtension);
inventory::collect!(RegistryFileOptions);
