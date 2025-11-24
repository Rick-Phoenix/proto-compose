use std::{fmt::Write, rc::Rc};

use syn::{punctuated::IterMut, ItemEnum, ItemStruct, MetaNameValue};

use crate::*;

pub enum ModuleItem2 {
  Message(MessageData),
  Oneof(OneofData),
}

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
    return name;
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
  module_attrs: ModuleAttrs,
  mut module: ItemMod,
) -> Result<ItemMod, Error> {
  let (brace, content) = if let Some((brace, content)) = module.content {
    (brace, content)
  } else {
    return Ok(module);
  };

  let mut mod_items: Vec<Item> = Vec::new();

  let mut messages_keys: Vec<Ident> = Vec::new();

  let mut oneofs: HashMap<Ident, OneofData> = HashMap::new();
  let mut messages: HashMap<Ident, MessageData> = HashMap::new();
  let mut enums: HashMap<Ident, EnumData> = HashMap::new();

  let mut proto_items: HashMap<Ident, ModuleItem2> = HashMap::new();
  let mut relational_map: HashMap<Ident, Ident> = HashMap::new();

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
          relational_map.insert(item_ident.clone(), nested_msg.clone());
        }

        proto_items.insert(item_ident, ModuleItem2::Message(message_data));
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

        let module_item = match enum_kind {
          EnumKind::Oneof => ModuleItem2::Oneof(parse_oneof(e)?),
          EnumKind::Enum => todo!(),
        };

        proto_items.insert(item_ident, module_item);
      }
      _ => {
        mod_items.push(item);
      }
    };
  }

  for msg in relational_map.keys() {
    register_full_name(msg, &relational_map, &mut messages);
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

#[derive(Debug)]
pub enum ItemKind<'a> {
  Message(MessageStruct<'a>),
  Enum(&'a mut ItemEnum),
  Oneof(OneofEnum<'a>),
}

#[derive(Debug)]
pub struct MessageStruct<'a> {
  pub tokens: &'a mut ItemStruct,
  pub reserved_numbers: ReservedNumbers,
  pub oneofs: HashMap<Ident, OneofEnum<'a>>,
  pub used_tags: Vec<i32>,
}

#[derive(Debug)]
pub struct OneofEnum<'a> {
  pub tokens: &'a mut ItemEnum,
  pub used_tags: Vec<i32>,
}

#[derive(Debug)]
pub struct ModuleItem<'a> {
  pub kind: ItemKind<'a>,
  pub name: Rc<str>,
}

impl<'a> ModuleItem<'a> {
  pub fn inject_attr(&mut self, attr: Attribute) {
    match &mut self.kind {
      ItemKind::Message(item) => item.tokens.attrs.push(attr),
      ItemKind::Enum(item) => item.attrs.push(attr),
      ItemKind::Oneof(item) => item.tokens.attrs.push(attr),
    }
  }

  pub fn get_ident(&self) -> &Ident {
    match &self.kind {
      ItemKind::Message(item) => &item.tokens.ident,
      ItemKind::Enum(item) => &item.ident,
      ItemKind::Oneof(item) => &item.tokens.ident,
    }
  }
}

pub enum DeriveKind {
  Message,
  Enum,
  Oneof,
}

pub struct ParentMessage {
  pub ident: Ident,
  pub name: Rc<str>,
}

enum EnumKind {
  Oneof,
  Enum,
}

pub fn process_module_items2(
  file_attribute: Attribute,
  items: &'_ mut Vec<Item>,
) -> Result<TopLevelItemsTokens, Error> {
  let mut module_items: HashMap<Ident, ModuleItem> = HashMap::new();
  let mut items_belonging_to_messages: HashMap<Ident, Vec<Ident>> = HashMap::new();

  let mut nested_items_map: HashMap<Ident, Rc<ParentMessage>> = HashMap::new();

  for item in items {
    let derive_kind = if let Some(kind) = get_derive_kind(item)? {
      kind
    } else {
      continue;
    };

    match item {
      Item::Struct(s) => {
        let mut name: Option<String> = None;
        let mut nested_items_list: Vec<Path> = Vec::new();
        let mut reserved_numbers = ReservedNumbers::default();
        let mut oneofs: Vec<Ident> = Vec::new();

        for attr in &s.attrs {
          if attr.path().is_ident("proto") {
            let metas = attr.parse_args::<PunctuatedParser<Meta>>().unwrap().inner;

            for meta in metas {
              match meta {
                Meta::List(list) => {
                  if list.path.is_ident("nested_messages") || list.path.is_ident("nested_enums") {
                    let paths = list.parse_args::<PunctuatedParser<Path>>()?.inner;

                    nested_items_list.extend(paths);
                  } else if list.path.is_ident("reserved_numbers") {
                    let numbers = list.parse_args::<ReservedNumbers>()?;

                    reserved_numbers = numbers;
                  }
                }
                Meta::NameValue(nv) => {
                  if nv.path.is_ident("name") {
                    name = Some(extract_string_lit(&nv.value)?);
                  }
                }
                _ => {}
              }
            }
          }
        }

        let mut used_tags: Vec<i32> = Vec::new();

        for field in process_struct_fields(s)? {
          for attr in &field.attrs {
            for meta in get_proto_args(attr)? {
              match meta {
                Meta::NameValue(nv) => {
                  if nv.path.is_ident("tag") {
                    let tag = extract_i32(&nv.value)?;

                    used_tags.push(tag);
                  }
                }
                Meta::Path(path) => {
                  if path.is_ident("oneof") {
                    let field_type = extract_type(&field.ty)?;

                    if !field_type.is_option() {
                      return Err(spanned_error!(path, "Oneofs must be wrapped in Option"));
                    }

                    oneofs.push(field_type.path().require_ident()?.clone());
                  }
                }
                _ => {}
              }
            }
          }
        }

        if !matches!(derive_kind, DeriveKind::Message) {
          panic!("The Message derive can only be used on structs");
        }

        let name: Rc<str> = if let Some(name_override) = name {
          name_override.into()
        } else {
          let inferred_name = s.ident.to_string();

          let name_attr: Attribute = parse_quote! { #[proto(name = #inferred_name)] };
          s.attrs.push(name_attr);

          inferred_name.into()
        };

        if !oneofs.is_empty() {
          items_belonging_to_messages.insert(s.ident.clone(), oneofs);
        }

        if !nested_items_list.is_empty() {
          let parent_message_info: Rc<ParentMessage> = ParentMessage {
            ident: s.ident.clone(),
            name: name.clone(),
          }
          .into();

          for nested_item in nested_items_list {
            let nested_item_ident = nested_item.require_ident()?;

            nested_items_map.insert(nested_item_ident.clone(), parent_message_info.clone());
          }
        }

        let message_ident = s.ident.clone();

        let message_struct = MessageStruct {
          tokens: s,
          oneofs: Default::default(),
          reserved_numbers,
          used_tags,
        };

        module_items.insert(
          message_ident,
          ModuleItem {
            name,
            kind: ItemKind::Message(message_struct),
          },
        );
      }
      Item::Enum(e) => {
        let mut name: Option<String> = None;

        for attr in &e.attrs {
          if attr.path().is_ident("proto") {
            let metas = attr.parse_args::<PunctuatedParser<Meta>>().unwrap().inner;

            for meta in metas {
              if let Meta::NameValue(nv) = meta
                && nv.path.is_ident("name") {
                  name = Some(extract_string_lit(&nv.value)?);
                }
            }
          }
        }

        let name: Rc<str> = if let Some(name_override) = name {
          name_override.into()
        } else {
          let inferred_name = e.ident.to_string();

          let name_attr: Attribute = parse_quote! { #[proto(name = #inferred_name)] };
          e.attrs.push(name_attr);

          inferred_name.into()
        };

        let enum_ident = e.ident.clone();

        let item_kind = match derive_kind {
          DeriveKind::Oneof => ItemKind::Oneof(OneofEnum {
            tokens: e,
            used_tags: vec![],
          }),
          DeriveKind::Enum => ItemKind::Enum(e),
          _ => {
            panic!("Cannot be a stuct");
          }
        };

        module_items.insert(
          enum_ident,
          ModuleItem {
            kind: item_kind,
            name,
          },
        );
      }
      _ => {}
    }
  }

  for (msg_ident, nested_items) in items_belonging_to_messages {
    let mut msg = module_items
      .remove(&msg_ident)
      .expect("could not find message in map");

    let mut msg_data = if let ItemKind::Message(msg) = &mut msg.kind {
      msg
    } else {
      panic!()
    };

    for nested in nested_items {
      let item = module_items
        .remove(&nested)
        .expect("could not find item in map");

      if let ItemKind::Oneof(mut oneof) = item.kind {
        for variant_res in process_enum_variants(&mut oneof.tokens) {
          let variant = variant_res?;

          for attr in &variant.attrs {
            if let Some(tag) = find_tag_attribute(attr)? {
              msg_data.used_tags.push(tag);
            }
          }
        }

        msg_data.oneofs.insert(nested, oneof);
      }
    }

    eprintln!("{:#?}", msg_data.used_tags);

    module_items.insert(msg_ident, msg);
  }

  let mut top_level_messages = TokenStream2::new();
  let mut top_level_enums = TokenStream2::new();

  for (item_ident, mut item) in module_items.into_iter() {
    item.inject_attr(file_attribute.clone());

    if let Some(parent_message) = nested_items_map.get(item.get_ident()) {
      let parent_message_ident = &parent_message.ident;

      let mut ancestors = vec![parent_message];
      let mut current_message = parent_message_ident;

      while let Some(parent) = nested_items_map.get(current_message) {
        ancestors.push(parent);
        current_message = &parent.ident;
      }

      let mut full_name = String::new();

      for ancestor in ancestors.iter().rev() {
        let ancestor_name = &ancestor.name;
        write!(full_name, "{ancestor_name}.").unwrap();
      }

      full_name.push_str(&item.name);

      let full_name_attr: Attribute = parse_quote! { #[proto(full_name = #full_name)] };

      item.inject_attr(full_name_attr);
    } else {
      let item_ident = item.get_ident();

      match item.kind {
        ItemKind::Message(_) => top_level_messages.extend(quote! { #item_ident::to_message(), }),
        ItemKind::Enum(_) => top_level_enums.extend(quote! { #item_ident::to_enum() }),
        ItemKind::Oneof(_) => {}
      }
    }

    if let ItemKind::Message(mut msg) = item.kind {
      let ranges = msg.reserved_numbers.build_unavailable_ranges(msg.used_tags);

      let mut tag_allocator = TagAllocator::new(&ranges.0);

      for field in process_struct_fields(msg.tokens)? {
        let tag = 'tag: {
          for attr in &field.attrs {
            if let Some(tag) = find_tag_attribute(attr)? {
              break 'tag tag;
            }
          }

          tag_allocator.next_tag()
        };

        eprintln!("Name: {}, tag: {tag}", field.ident.as_ref().unwrap());
      }
    }
  }

  Ok(TopLevelItemsTokens {
    top_level_messages,
    top_level_enums,
  })
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

pub fn get_derive_kind(item: &Item) -> Result<Option<DeriveKind>, Error> {
  let attrs = match item {
    Item::Struct(s) => &s.attrs,
    Item::Enum(e) => &e.attrs,
    _ => return Ok(None),
  };

  for attr in attrs {
    if attr.path().is_ident("derive") {
      let derives = attr
        .meta
        .require_list()?
        .parse_args::<PunctuatedParser<Path>>()?
        .inner;

      for path in derives {
        if path.is_ident("Message") {
          return Ok(Some(DeriveKind::Message));
        } else if path.is_ident("Enum") {
          return Ok(Some(DeriveKind::Enum));
        } else if path.is_ident("Oneof") {
          return Ok(Some(DeriveKind::Oneof));
        }
      }

      return Ok(None);
    }
  }

  Ok(None)
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

pub fn has_derive() -> Result<Option<DeriveKind>, Error> {
  let attrs = match item {
    Item::Struct(s) => &s.attrs,
    Item::Enum(e) => &e.attrs,
    _ => return Ok(None),
  };

  for attr in attrs {
    if attr.path().is_ident("derive") {
      let derives = attr
        .meta
        .require_list()?
        .parse_args::<PunctuatedParser<Path>>()?
        .inner;

      for path in derives {
        if path.is_ident("Message") {
          return Ok(Some(DeriveKind::Message));
        } else if path.is_ident("Enum") {
          return Ok(Some(DeriveKind::Enum));
        } else if path.is_ident("Oneof") {
          return Ok(Some(DeriveKind::Oneof));
        }
      }

      return Ok(None);
    }
  }

  Ok(None)
}
