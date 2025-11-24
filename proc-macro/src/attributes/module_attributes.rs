use syn::{punctuated::IterMut, ItemEnum, ItemStruct, MetaNameValue};

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

pub fn process_module_items(
  package_attr: &Attribute,
  mut module: ItemMod,
) -> Result<ItemMod, Error> {
  let (brace, content) = if let Some((brace, content)) = module.content {
    (brace, content)
  } else {
    return Ok(module);
  };

  let mut mod_items: Vec<Item> = Vec::new();

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
          mod_items.push(Item::Struct(s));
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

        messages.insert(item_ident, message_data);
      }
      Item::Enum(e) => {
        let derives = Derives::new(&e.attrs)?;

        let enum_kind = if let Some(kind) = derives.enum_kind() {
          kind
        } else {
          mod_items.push(Item::Enum(e));
          continue;
        };

        let item_ident = e.ident.clone();

        match enum_kind {
          EnumKind::Oneof => {
            oneofs.insert(item_ident, parse_oneof(e)?);
          }
          EnumKind::Enum => {
            enums.insert(item_ident, parse_enum(e)?);
          }
        };
      }
      _ => {
        mod_items.push(item);
      }
    };
  }

  for msg in messages_relational_map.keys() {
    register_full_name(msg, &messages_relational_map, &mut messages);
  }

  for (_, mut msg) in messages {
    process_message_from_module(&mut msg, &mut oneofs, package_attr)?;

    mod_items.push(Item::Struct(msg.into()));
  }

  for (_, mut enum_) in enums {
    process_enum_from_module(&mut enum_, package_attr)?;

    mod_items.push(Item::Enum(enum_.into()));
  }

  for (_, oneof) in oneofs {
    mod_items.push(Item::Enum(oneof.into()));
  }

  module.content = Some((brace, mod_items));

  Ok(module)
}

pub struct ModuleAttrs {
  pub file: String,
  pub package: String,
}

pub struct TopLevelItemsTokens {
  pub top_level_messages: TokenStream2,
  pub top_level_enums: TokenStream2,
}

pub enum EnumKind {
  Oneof,
  Enum,
}

fn find_tag_attribute(attr: &Attribute) -> Result<Option<i32>, Error> {
  if attr.path().is_ident("proto") {
    let args = attr.parse_args::<PunctuatedParser<Meta>>()?;

    for meta in &args.inner {
      if let Meta::NameValue(nv) = meta && nv.path.is_ident("tag") {
        let tag = extract_i32(&nv.value)?;

        return Ok(Some(tag));
      }
    }
  }

  Ok(None)
}

fn process_enum_variants(
  target_enum: &mut ItemEnum,
) -> impl Iterator<Item = Result<&mut Variant, Error>> {
  target_enum.variants.iter_mut().map(|variant| {
    if let Fields::Unnamed(fields) = &mut variant.fields && fields.unnamed.len() == 1 {
      Ok(variant)
    } else {
      Err(spanned_error!(
        variant.ident.clone(),
        "Must be an enum variant with a single unnamed field"
      ))
    }
  })
}

pub fn process_struct_fields(
  target_struct: &'_ mut ItemStruct,
) -> Result<IterMut<'_, Field>, Error> {
  if let Fields::Named(fields) = &mut target_struct.fields {
    Ok(fields.named.iter_mut())
  } else {
    Err(spanned_error!(
      target_struct.ident.clone(),
      "Must be a struct with named fields"
    ))
  }
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
