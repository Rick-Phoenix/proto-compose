use syn::MetaNameValue;

use crate::*;

fn register_full_name(
  msg: &Ident,
  relational_map: &HashMap<Ident, Ident>,
  messages_map: &mut HashMap<Ident, MessageData>,
) {
  let is_registered = messages_map.get(msg).unwrap().full_name.get().is_some();

  if !is_registered {
    let short_name = messages_map.get(msg).unwrap().name.clone();

    if let Some(parent) = relational_map.get(msg) {
      let parent_name = get_full_name(parent, relational_map, messages_map);

      let full_name = format!("{parent_name}.{short_name}");

      let _ = messages_map.get_mut(msg).unwrap().full_name.set(full_name);
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
    let msg_data = messages_map.get(msg).unwrap();
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
    match item {
      Item::Struct(s) => {
        let derives = Derives::new(&s.attrs)?;

        if !derives.contains("Message") {
          mod_items.push(ModuleItem::Raw(Item::Struct(s).into()));
          continue;
        }

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
        let derives = Derives::new(&e.attrs)?;

        let enum_kind = if let Some(kind) = derives.enum_kind() {
          kind
        } else {
          mod_items.push(ModuleItem::Raw(Item::Enum(e).into()));
          continue;
        };

        let item_ident = e.ident.clone();

        match enum_kind {
          EnumKind::Oneof => {
            mod_items.push(ModuleItem::Oneof(item_ident.clone()));
            oneofs.insert(item_ident, parse_oneof(e)?);
          }
          EnumKind::Enum => {
            mod_items.push(ModuleItem::Enum(item_ident.clone()));
            enums.insert(item_ident, parse_enum(e)?);
          }
        };
      }
      _ => {
        mod_items.push(ModuleItem::Raw(item.into()));
      }
    };
  }

  let mut top_level_enums = TokenStream2::new();
  let mut top_level_messages = TokenStream2::new();

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
    let parent_message = if let Some(parent) = enums_relational_map.get(ident) {
      let parent = messages.get(parent).expect("Message not found");

      Some(parent.full_name.get().unwrap_or(&parent.name).clone())
    } else {
      top_level_enums.extend(quote! { #ident::to_enum(), });
      None
    };

    process_enum_from_module(enum_, parent_message, &package_attr)?;
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

pub enum EnumKind {
  Oneof,
  Enum,
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

pub struct Derives {
  pub list: Vec<Path>,
}

impl Derives {
  pub fn contains(&self, value: &str) -> bool {
    for item in &self.list {
      let last_segment = item.segments.last().unwrap();

      if last_segment.ident == value {
        return true;
      }
    }

    false
  }

  pub fn enum_kind(&self) -> Option<EnumKind> {
    for item in &self.list {
      let last_segment = item.segments.last().unwrap();

      if last_segment.ident == "Oneof" {
        return Some(EnumKind::Oneof);
      } else if last_segment.ident == "Enum" {
        return Some(EnumKind::Enum);
      }
    }

    None
  }
}

impl Derives {
  pub fn new(attrs: &[Attribute]) -> Result<Self, Error> {
    let mut list: Vec<Path> = Vec::new();

    for attr in attrs {
      if attr.path().is_ident("derive") {
        list.extend(
          attr
            .meta
            .require_list()?
            .parse_args::<PunctuatedParser<Path>>()?
            .inner,
        );
      }
    }

    Ok(Self { list })
  }
}
