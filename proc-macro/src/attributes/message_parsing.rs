use std::cell::OnceCell;

use syn::FieldsNamed;

use crate::*;

pub struct MessageData {
  pub tokens: StructRaw,
  pub fields: Vec<FieldData>,
  pub reserved_names: ReservedNames,
  pub reserved_numbers: ReservedNumbers,
  pub options: ProtoOptions,
  pub name: String,
  pub full_name: OnceCell<String>,
  pub nested_messages: Vec<Ident>,
  pub nested_enums: Vec<Ident>,
  pub used_tags: Vec<i32>,
  pub parent_message: Option<Rc<str>>,
}

impl From<MessageData> for ItemStruct {
  fn from(value: MessageData) -> Self {
    let fields: Punctuated<Field, Token![,]> = value
      .fields
      .into_iter()
      .map(|field| field.field_raw)
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

pub struct FieldData {
  pub field_raw: Field,
  pub data: FieldAttrs,
  pub type_: Path,
}

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
    return Err(spanned_error!(ident, "Must be a struct with named fields"));
  };

  let MessageAttrs {
    reserved_names,
    reserved_numbers,
    options,
    name,
    nested_enums,
    nested_messages,
  } = process_message_attrs(&ident, &attrs)?;

  let mut used_tags: Vec<i32> = Vec::new();
  let mut fields_data: Vec<FieldData> = Vec::new();

  for field in fields {
    let data = if let Some(field_attrs) = process_field_attrs(&ident, &attrs)? {
      field_attrs
    } else {
      continue;
    };

    let field_type = extract_type(&field.ty)?;

    if data.is_oneof {
      if !field_type.is_option() {
        return Err(spanned_error!(
          &field.ty,
          "Oneof fields must be wrapped in Option"
        ));
      }
    }

    if let Some(tag) = data.tag {
      used_tags.push(tag);
    }

    fields_data.push(FieldData {
      type_: field_type.path().clone(),
      field_raw: field,
      data,
    });
  }

  Ok(MessageData {
    tokens: StructRaw {
      attrs,
      vis,
      ident,
      generics,
    },
    parent_message: None,
    used_tags,
    fields: fields_data,
    reserved_names,
    reserved_numbers,
    options,
    name,
    nested_messages,
    nested_enums,
    full_name: OnceCell::new(),
  })
}
