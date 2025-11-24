use std::cell::OnceCell;

use syn::FieldsNamed;

use crate::*;

#[derive(Debug)]
pub struct MessageData {
  pub tokens: StructRaw,
  pub fields: Vec<FieldData>,
  pub reserved_names: ReservedNames,
  pub reserved_numbers: ReservedNumbers,
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
    let fields: Punctuated<Field, Token![,]> =
      value.fields.into_iter().map(|field| field.tokens).collect();

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
  pub is_oneof: bool,
  pub type_: Path,
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
    return Err(spanned_error!(ident, "Must be a struct with named fields"));
  };

  let ModuleMessageAttrs {
    reserved_names,
    reserved_numbers,
    name,
    nested_enums,
    nested_messages,
  } = process_module_message_attrs(&ident, &attrs)?;

  let mut used_tags: Vec<i32> = Vec::new();
  let mut oneofs: Vec<Ident> = Vec::new();
  let mut fields_data: Vec<FieldData> = Vec::new();

  for field in fields {
    let ModuleFieldAttrs {
      tag,
      name,
      is_oneof,
    } = if let Some(field_attrs) =
      process_module_field_attrs(field.ident.as_ref().unwrap(), &field.attrs)?
    {
      field_attrs
    } else {
      continue;
    };

    let field_type = extract_type(&field.ty)?;

    if is_oneof {
      if !field_type.is_option() {
        return Err(spanned_error!(
          &field.ty,
          "Oneof fields must be wrapped in Option"
        ));
      }

      oneofs.push(field_type.path().require_ident()?.clone());
    }

    if let Some(tag) = tag {
      used_tags.push(tag);
    }

    fields_data.push(FieldData {
      type_: field_type.path().clone(),
      tokens: field,
      tag,
      name,
      is_oneof,
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
    reserved_names,
    reserved_numbers,
    name,
    nested_messages,
    nested_enums,
    full_name: OnceCell::new(),
    oneofs,
  })
}
