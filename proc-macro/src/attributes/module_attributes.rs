use syn::MetaNameValue;

use crate::*;

fn register_full_name(
  msg: &Ident,
  relational_map: &HashMap<Ident, Ident>,
  messages_map: &mut HashMap<Ident, MessageData>,
) {
  let is_registered = messages_map
    .get(msg)
    .expect("could not find message")
    .full_name
    .get()
    .is_some();

  if !is_registered {
    let short_name = messages_map
      .get(msg)
      .expect("could not find message")
      .name
      .clone();

    if let Some(parent) = relational_map.get(msg) {
      let parent_name = get_full_name(parent, relational_map, messages_map);

      let full_name = format!("{parent_name}.{short_name}");

      let _ = messages_map
        .get_mut(msg)
        .expect("could not find message")
        .full_name
        .set(full_name);
    }
  }
}

fn get_full_name(
  msg: &Ident,
  relational_map: &HashMap<Ident, Ident>,
  messages_map: &mut HashMap<Ident, MessageData>,
) -> String {
  let mut is_full = false;

  let name = {
    let msg_data = messages_map.get(msg).expect("could not find message");
    if let Some(full_name) = msg_data.full_name.get() {
      is_full = true;
      full_name.clone()
    } else {
      msg_data.name.clone()
    }
  };

  if is_full {
    name
  } else {
    match relational_map.get(msg) {
      None => name,
      Some(parent) => {
        let parent_name = get_full_name(parent, relational_map, messages_map);

        let full_name = format!("{parent_name}.{name}");

        let _ = messages_map
          .get_mut(msg)
          .unwrap()
          .full_name
          .set(full_name.clone());

        full_name
      }
    }
  }
}

pub enum ModuleItem {
  Raw(Box<Item>),
  Oneof(Ident),
  Message(Ident),
  Enum(Ident),
}

pub fn process_module_items(
  module_attrs: ModuleAttrs,
  mut module: ItemMod,
) -> Result<ItemMod, Error> {
  let (brace, content) = if let Some((brace, content)) = module.content {
    (brace, content)
  } else {
    return Ok(module);
  };

  let ModuleAttrs { file, package } = module_attrs;

  let package_attr: Attribute = parse_quote! { #[proto(file = #file, package = #package)] };

  let mut mod_items: Vec<ModuleItem> = Vec::new();

  let mut oneofs: HashMap<Ident, OneofData> = HashMap::new();
  let mut messages: HashMap<Ident, MessageData> = HashMap::new();
  let mut enums: HashMap<Ident, EnumData> = HashMap::new();

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

        let message_data = parse_message(s)?;

        for nested_msg in &message_data.nested_messages {
          messages_relational_map.insert(item_ident.clone(), nested_msg.clone());
        }

        for nested_enum in &message_data.nested_enums {
          enums_relational_map.insert(item_ident.clone(), nested_enum.clone());
        }

        mod_items.push(ModuleItem::Message(item_ident.clone()));
        messages.insert(item_ident, message_data);
      }
      Item::Enum(e) => {
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
          _ => unreachable!(),
        };
      }
      _ => {
        mod_items.push(ModuleItem::Raw(item.into()));
      }
    };
  }

  let mut top_level_enums = TokenStream2::new();
  let mut top_level_messages = TokenStream2::new();

  eprintln!("{:#?}", messages);

  for msg in messages_relational_map.keys() {
    register_full_name(msg, &messages_relational_map, &mut messages);
  }

  for (ident, msg) in messages.iter_mut() {
    let is_top_level = !messages_relational_map.contains_key(ident);

    if is_top_level {
      top_level_messages.extend(quote! { #ident::to_message(), });
    }

    process_message_from_module(msg, &mut oneofs, &package_attr)?;
  }

  for (ident, enum_) in enums.iter_mut() {
    if let Some(parent) = enums_relational_map.get(ident) {
      let parent = messages.get(parent).expect("Message not found");

      let parent_message_name = parent.full_name.get().unwrap_or(&parent.name);
      let enum_proto_name = &enum_.name;

      let full_name = format!("{parent_message_name}.{enum_proto_name}");

      let full_name_attr: Attribute = parse_quote!(#[proto(full_name = #full_name)]);

      enum_.tokens.attrs.push(full_name_attr);
    } else {
      top_level_enums.extend(quote! { #ident::to_enum(), });
    };

    enum_.tokens.attrs.push(package_attr.clone());
  }

  let mut processed_items: Vec<Item> = Vec::new();

  for item in mod_items {
    processed_items.push(match item {
      ModuleItem::Raw(item) => *item,
      ModuleItem::Oneof(ident) => {
        Item::Enum(oneofs.remove(&ident).expect("Oneof not found").into())
      }
      ModuleItem::Message(ident) => {
        Item::Struct(messages.remove(&ident).expect("Message not found").into())
      }
      ModuleItem::Enum(ident) => Item::Enum(enums.remove(&ident).expect("Enum not found").into()),
    });
  }

  let aggregator_fn: ItemFn = parse_quote! {
    pub fn proto_file() -> ProtoFile {
      let mut file = ProtoFile {
        name: #file.into(),
        package: #package.into(),
        ..Default::default()
      };

      file.add_messages([ #top_level_messages ]);
      file.add_enums([ #top_level_enums ]);

      file
    }
  };

  processed_items.push(Item::Fn(aggregator_fn));

  module.content = Some((brace, processed_items));

  Ok(module)
}

pub struct ModuleAttrs {
  pub file: String,
  pub package: String,
}

impl Parse for ModuleAttrs {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let mut file: Option<String> = None;
    let mut package: Option<String> = None;

    let args = Punctuated::<MetaNameValue, Token![,]>::parse_terminated(input)?;

    for arg in args {
      if arg.path.is_ident("file") {
        file = Some(extract_string_lit(&arg.value)?);
      } else if arg.path.is_ident("package") {
        package = Some(extract_string_lit(&arg.value)?);
      }
    }

    let file = file.ok_or(error!(Span::call_site(), "File attribute is missing"))?;
    let package = package.ok_or(error!(Span::call_site(), "Package attribute is missing"))?;

    Ok(ModuleAttrs { file, package })
  }
}

pub enum ItemKind {
  Message,
  Enum,
  Oneof,
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
              item,
              "proto_message can only be used on a struct"
            ));
          }

          return Ok(Some(Self::Message));
        }
        "proto_enum" => {
          if is_struct {
            return Err(spanned_error!(
              item,
              "proto_enum can only be used on an enum"
            ));
          }
          return Ok(Some(Self::Enum));
        }
        "proto_oneof" => {
          if is_struct {
            return Err(spanned_error!(
              item,
              "proto_oneof can only be used on an enum"
            ));
          }

          return Ok(Some(Self::Oneof));
        }
        _ => {}
      };
    }

    Ok(None)
  }
}
