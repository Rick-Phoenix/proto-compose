use crate::*;

// We collect the items in this so that we can retain the order in the output
pub enum ModuleItem {
  Raw(Box<Item>),
  Oneof(Ident),
  Message(Ident),
  Enum(Ident),
}

// The module macro currently does this:
// - It processes nested items and emits their full names
// - It processes tags for oneofs and messages
// - It injects the package and module attributes
pub fn process_module_items(
  module_attrs: ModuleAttrs,
  mut module: ItemMod,
) -> Result<ItemMod, Error> {
  let (brace, content) = if let Some((brace, content)) = module.content {
    (brace, content)
  } else {
    return Ok(module);
  };

  let mut mod_items: Vec<ModuleItem> = Vec::new();

  let mut oneofs: HashMap<Ident, OneofData> = HashMap::new();
  let mut messages: HashMap<Ident, MessageData> = HashMap::new();
  let mut enums: HashMap<Ident, EnumData> = HashMap::new();
  let mut services: Vec<Ident> = Vec::new();
  let mut extensions: Vec<Ident> = Vec::new();

  let mut messages_relational_map: HashMap<Ident, Ident> = HashMap::new();
  let mut enums_relational_map: HashMap<Ident, Ident> = HashMap::new();

  for item in content {
    let item_kind = if let Some(kind) = ItemKind::detect(&item)? {
      kind
    } else {
      mod_items.push(ModuleItem::Raw(item.into()));
      continue;
    };

    match item {
      Item::Struct(s) => {
        let item_ident = s.ident.clone();

        match item_kind {
          ItemKind::Extension => {
            extensions.push(item_ident);

            mod_items.push(ModuleItem::Raw(Item::Struct(s).into()));
            continue;
          }
          ItemKind::Message => {
            let message_data = parse_message(s)?;

            for nested_msg_ident in &message_data.nested_messages {
              messages_relational_map.insert(nested_msg_ident.clone(), item_ident.clone());
            }

            for nested_enum_ident in &message_data.nested_enums {
              enums_relational_map.insert(nested_enum_ident.clone(), item_ident.clone());
            }

            mod_items.push(ModuleItem::Message(item_ident.clone()));
            messages.insert(item_ident, message_data);
          }
          _ => unreachable!(),
        };
      }
      Item::Enum(mut e) => {
        let item_ident = e.ident.clone();

        match item_kind {
          ItemKind::Oneof => {
            mod_items.push(ModuleItem::Oneof(item_ident.clone()));
            oneofs.insert(item_ident, parse_oneof(e)?);
          }
          ItemKind::Enum => {
            mod_items.push(ModuleItem::Enum(item_ident.clone()));
            enums.insert(item_ident, parse_enum(e)?);
          }
          ItemKind::Service => {
            let package = &module_attrs.package;

            e.attrs.push(parse_quote!(#[proto(package = #package)]));

            mod_items.push(ModuleItem::Raw(Item::Enum(e).into()));
            services.push(item_ident);
          }
          _ => unreachable!(),
        };
      }
      _ => {
        mod_items.push(ModuleItem::Raw(item.into()));
      }
    };
  }

  let mut top_level_enums: Vec<Ident> = Vec::new();
  let mut top_level_messages: Vec<Ident> = Vec::new();

  for nested_msg_ident in messages_relational_map.keys() {
    register_full_name(nested_msg_ident, &messages_relational_map, &mut messages)?;
  }

  for (ident, msg) in messages.iter_mut() {
    let is_top_level = !messages_relational_map.contains_key(ident);

    if is_top_level {
      top_level_messages.push(ident.clone());
    }

    process_message_from_module(msg, &mut oneofs, &module_attrs)?;
  }

  for (ident, enum_) in enums.iter_mut() {
    if let Some(parent) = enums_relational_map.get(ident) {
      let parent = messages.get(parent).ok_or(spanned_error!(
        parent,
        format!("Failed to find the data for the message `{parent}`")
      ))?;

      let parent_message_name = parent.full_name.get().unwrap_or(&parent.name);
      let enum_proto_name = &enum_.name;

      let full_name = format!("{parent_message_name}.{enum_proto_name}");
      let full_name_attr: Attribute = parse_quote!(#[proto(full_name = #full_name)]);

      enum_.tokens.attrs.push(full_name_attr);
    } else {
      top_level_enums.push(ident.clone());
    };

    enum_.tokens.attrs.push(module_attrs.as_attribute());
  }

  let mut processed_items: Vec<Item> = Vec::new();

  for item in mod_items {
    processed_items.push(match item {
      ModuleItem::Raw(item) => *item,
      ModuleItem::Oneof(ident) => Item::Enum(
        oneofs
          .remove(&ident)
          .ok_or(spanned_error!(
            &ident,
            format!("Failed to find the data for the oneof `{ident}`")
          ))?
          .into(),
      ),
      ModuleItem::Message(ident) => Item::Struct(
        messages
          .remove(&ident)
          .ok_or(spanned_error!(
            &ident,
            format!("Failed to find the data for the message `{ident}`")
          ))?
          .into(),
      ),
      ModuleItem::Enum(ident) => Item::Enum(
        enums
          .remove(&ident)
          .ok_or(spanned_error!(
            &ident,
            format!("Failed to find the data for the enum `{ident}`")
          ))?
          .into(),
      ),
    });
  }

  let ModuleAttrs { file, package, .. } = module_attrs;

  let aggregator_fn: ItemFn = parse_quote! {
    pub fn proto_file() -> ::prelude::ProtoFile {
      let mut file = ::prelude::ProtoFile::new(#file, #package);

      let extensions = vec![ #(#extensions::as_proto_extension()),* ];

      if !extensions.is_empty() {
        file.add_extensions(extensions);
      }

      file.add_messages([ #(#top_level_messages::proto_schema()),* ]);
      file.add_enums([ #(#top_level_enums::proto_schema()),* ]);
      file.add_services([ #(#services::proto_schema()),* ]);

      file
    }
  };

  processed_items.push(Item::Fn(aggregator_fn));

  module.content = Some((brace, processed_items));

  Ok(module)
}

fn register_full_name(
  msg: &Ident,
  relational_map: &HashMap<Ident, Ident>,
  messages_map: &mut HashMap<Ident, MessageData>,
) -> Result<(), Error> {
  let target = messages_map.get(msg).ok_or(spanned_error!(
    msg,
    format!("Could not find the data for the message `{msg}`")
  ))?;

  let has_full_name = target.full_name.get().is_some();

  if !has_full_name {
    let short_name = target.name.clone();

    if let Some(parent) = relational_map.get(msg) {
      let parent_name = get_full_name(parent, relational_map, messages_map)?;

      let full_name = format!("{parent_name}.{short_name}");

      let _ = messages_map
        .get_mut(msg)
        .ok_or(spanned_error!(
          msg,
          format!("Could not find the data for the message `{msg}`")
        ))?
        .full_name
        .set(full_name);
    }
  }

  Ok(())
}

fn get_full_name(
  msg: &Ident,
  relational_map: &HashMap<Ident, Ident>,
  messages_map: &mut HashMap<Ident, MessageData>,
) -> Result<String, Error> {
  let mut found_full_name = false;

  let name = {
    let msg_data = messages_map.get(msg).ok_or(spanned_error!(
      msg,
      format!("Could not find the data for the message `{msg}`")
    ))?;

    if let Some(full_name) = msg_data.full_name.get() {
      found_full_name = true;
      full_name.clone()
    } else {
      msg_data.name.clone()
    }
  };

  if found_full_name {
    Ok(name)
  } else {
    match relational_map.get(msg) {
      None => Ok(name),
      Some(parent) => {
        let parent_name = get_full_name(parent, relational_map, messages_map)?;

        let full_name = format!("{parent_name}.{name}");

        let _ = messages_map
          .get_mut(msg)
          // Safe now since we know we have it
          .unwrap()
          .full_name
          .set(full_name.clone());

        Ok(full_name)
      }
    }
  }
}

// We use this to discriminate the kind of item we have before we process them
pub enum ItemKind {
  Message,
  Enum,
  Oneof,
  Service,
  Extension,
}

impl ItemKind {
  pub fn detect(item: &Item) -> Result<Option<Self>, Error> {
    let mut is_struct = false;

    let attrs = match item {
      Item::Struct(s) => {
        is_struct = true;
        &s.attrs
      }
      Item::Enum(e) => &e.attrs,
      _ => return Ok(None),
    };

    for attr in attrs {
      let ident = if let Some(path) = attr.path().segments.last() {
        path.ident.to_string()
      } else {
        continue;
      };

      match ident.as_str() {
        "proto_message" => {
          if !is_struct {
            return Err(spanned_error!(
              attr,
              "proto_message can only be used on a struct"
            ));
          }

          return Ok(Some(Self::Message));
        }
        "proto_extension" => {
          if !is_struct {
            return Err(spanned_error!(
              attr,
              "proto_extension can only be used on a struct"
            ));
          }

          return Ok(Some(Self::Extension));
        }
        "proto_enum" => {
          if is_struct {
            return Err(spanned_error!(
              attr,
              "proto_enum can only be used on an enum"
            ));
          }
          return Ok(Some(Self::Enum));
        }
        "proto_oneof" => {
          if is_struct {
            return Err(spanned_error!(
              attr,
              "proto_oneof can only be used on an enum"
            ));
          }

          return Ok(Some(Self::Oneof));
        }
        "proto_service" => {
          if is_struct {
            return Err(spanned_error!(
              attr,
              "proto_service can only be used on an enum"
            ));
          }

          return Ok(Some(Self::Service));
        }
        _ => {}
      };
    }

    Ok(None)
  }
}
