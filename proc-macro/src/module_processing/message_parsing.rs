use std::cell::OnceCell;

use syn::FieldsNamed;

use crate::*;

#[derive(Debug)]
pub struct MessageData {
  pub tokens: StructRaw,
  pub fields: Vec<FieldData>,
  pub reserved_numbers: ReservedNumbers,
  pub reserved_names: Vec<String>,
  pub name: String,
  pub full_name: OnceCell<String>,
  pub nested_messages: Vec<Ident>,
  pub nested_enums: Vec<Ident>,
  pub oneofs: Vec<Ident>,
  pub used_tags: Vec<i32>,
}

impl MessageData {
  pub fn inject_attr(&mut self, attr: Attribute) {
    self.tokens.attrs.push(attr);
  }
}

impl From<MessageData> for ItemStruct {
  fn from(value: MessageData) -> Self {
    let fields: Punctuated<Field, Token![,]> = value
      .fields
      .into_iter()
      .map(|field| field.tokens)
      .collect();

    let fields_named = FieldsNamed {
      named: fields,
      brace_token: Brace::default(),
    };

    Self {
      attrs: value.tokens.attrs,
      vis: value.tokens.vis,
      struct_token: Struct::default(),
      ident: value.tokens.ident,
      generics: value.tokens.generics,
      fields: Fields::Named(fields_named),
      semi_token: None,
    }
  }
}

#[derive(Debug)]
pub struct FieldData {
  pub tokens: Field,
  pub tag: Option<i32>,
  pub name: String,
  pub oneof_ident: Option<Ident>,
  pub is_ignored: bool,
}

impl FieldData {
  pub fn inject_attr(&mut self, attr: Attribute) {
    self.tokens.attrs.push(attr);
  }
}

#[derive(Debug)]
pub struct StructRaw {
  pub attrs: Vec<Attribute>,
  pub vis: Visibility,
  pub ident: Ident,
  pub generics: Generics,
}

pub fn parse_message(msg: ItemStruct) -> Result<MessageData, Error> {
  let ItemStruct {
    attrs,
    vis,
    ident,
    generics,
    fields,
    ..
  } = msg;

  let fields = if let Fields::Named(fields) = fields {
    fields.named
  } else {
    return Err(spanned_error!(ident, "Expected a struct with named fields"));
  };

  let ModuleMessageAttrs {
    reserved_numbers,
    name,
    nested_enums,
    nested_messages,
    reserved_names,
  } = process_module_message_attrs(&ident, &attrs)?;

  let mut used_tags: Vec<i32> = Vec::new();
  let mut oneofs: Vec<Ident> = Vec::new();
  let mut fields_data: Vec<FieldData> = Vec::new();

  for field in fields {
    let mut oneof_ident: Option<Ident> = None;

    let field_ident = field
      .ident
      .as_ref()
      .ok_or(spanned_error!(&field, "Expected a named field"))?;

    let ModuleFieldAttrs {
      tag,
      name,
      is_ignored,
      oneof_info,
    } = process_module_field_attrs(field_ident, &field.attrs)?;

    if let Some(tag) = tag {
      used_tags.push(tag);
    }

    if let Some(info) = oneof_info {
      let mut oneof_path = match info.path {
        ItemPathEntry::Path(path) => path,
        _ => {
          let rust_type = TypeInfo::from_type(&field.ty)?;

          rust_type.inner().as_path().ok_or(spanned_error!(
            &field,
            "Could not infer the path to the oneof. Please set it manually"
          ))?
        }
      };

      let found_oneof_ident = oneof_path
        .segments
        .pop()
        .unwrap()
        .into_value()
        .ident;

      oneof_ident = Some(found_oneof_ident.clone());
      oneofs.push(found_oneof_ident);
    }

    fields_data.push(FieldData {
      oneof_ident,
      tokens: field,
      tag,
      name,
      is_ignored,
    });
  }

  Ok(MessageData {
    tokens: StructRaw {
      attrs,
      vis,
      ident,
      generics,
    },
    used_tags,
    fields: fields_data,
    reserved_numbers,
    name,
    nested_messages,
    nested_enums,
    full_name: OnceCell::new(),
    oneofs,
    reserved_names,
  })
}
