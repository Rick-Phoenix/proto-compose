use quote::format_ident;

use crate::*;

fn create_shadow_struct(item: &ItemStruct) -> ItemStruct {
  let item_fields = if let Fields::Named(fields) = &item.fields {
    fields.named.iter().map(|f| Field {
      attrs: vec![],
      vis: f.vis.clone(),
      mutability: syn::FieldMutability::None,
      ident: f.ident.clone(),
      colon_token: f.colon_token,
      ty: f.ty.clone(),
    })
  } else {
    unreachable!()
  };

  ItemStruct {
    attrs: vec![],
    vis: Visibility::Public(token::Pub::default()),
    struct_token: token::Struct::default(),
    ident: format_ident!("{}Proto", item.ident),
    generics: item.generics.clone(),
    fields: Fields::Named(syn::FieldsNamed {
      brace_token: token::Brace::default(),
      named: item_fields.collect(),
    }),
    semi_token: None,
  }
}

pub(crate) fn process_message_derive_shadow(
  item: &mut ItemStruct,
  message_attrs: MessageAttrs,
) -> Result<TokenStream2, Error> {
  let mut output_tokens = TokenStream2::new();

  let mut shadow_struct = create_shadow_struct(item);

  let ItemStruct {
    ident: orig_struct_name,
    fields,
    ..
  } = item;

  let mut fields_data: Vec<TokenStream2> = Vec::new();
  let mut conversion_body = TokenStream2::new();

  let orig_struct_fields = fields.iter_mut();
  let shadow_struct_fields = shadow_struct.fields.iter_mut();

  for (src_field, dst_field) in orig_struct_fields.zip(shadow_struct_fields) {
    let src_field_ident = src_field.ident.as_ref().expect("Expected named field");

    let field_attrs =
      if let Some(attrs) = process_derive_field_attrs(src_field_ident, &src_field.attrs)? {
        attrs
      } else {
        continue;
      };

    let src_field_type = TypeInfo::from_type(&src_field.ty)?;

    let field_tokens = process_field(dst_field, field_attrs, &src_field_type, OutputType::Change)?;

    fields_data.push(field_tokens);

    let conversion_call = src_field_type.rust_type.conversion_call();

    conversion_body.extend(quote! {
      #src_field_ident: value.#src_field_ident.#conversion_call,
    });
  }

  let schema_impls = create_schema_impls(orig_struct_name, &message_attrs, fields_data);

  output_tokens.extend(schema_impls);

  let shadow_struct_ident = &shadow_struct.ident;

  output_tokens.extend(quote! {
    #[derive(prost::Message, Clone, PartialEq)]
    #shadow_struct

    impl AsProtoType for #shadow_struct_ident {
      fn proto_type() -> ProtoType {
        <#orig_struct_name as AsProtoType>::proto_type()
      }
    }
  });

  let conversion_tokens = quote! {
    impl From<#shadow_struct_ident> for #orig_struct_name {
      fn from(value: #shadow_struct_ident) -> Self {
        Self {
          #conversion_body
        }
      }
    }
  };

  output_tokens.extend(conversion_tokens);

  Ok(output_tokens)
}

pub(crate) fn process_message_derive(item: &mut ItemStruct) -> Result<TokenStream2, Error> {
  let message_attrs = process_derive_message_attrs(&item.ident, &item.attrs)?;

  if message_attrs.direct {
    process_message_derive_direct(item, message_attrs)
  } else {
    process_message_derive_shadow(item, message_attrs)
  }
}

pub(crate) fn process_message_derive_direct(
  item: &mut ItemStruct,
  message_attrs: MessageAttrs,
) -> Result<TokenStream2, Error> {
  let mut output_tokens = TokenStream2::new();

  let prost_message_attr: Attribute = parse_quote!(#[derive(prost::Message, Clone, PartialEq)]);

  item.attrs.push(prost_message_attr);

  let ItemStruct {
    ident: struct_name,
    fields,
    ..
  } = item;

  let mut fields_data: Vec<TokenStream2> = Vec::new();

  for src_field in fields {
    let src_field_ident = src_field.ident.as_ref().expect("Expected named field");

    let field_attrs =
      if let Some(attrs) = process_derive_field_attrs(src_field_ident, &src_field.attrs)? {
        attrs
      } else {
        return Err(spanned_error!(
          src_field,
          "Fields cannot be ignored in a direct impl"
        ));
      };

    let type_info = TypeInfo::from_type(&src_field.ty)?;

    let field_tokens = process_field(src_field, field_attrs, &type_info, OutputType::Keep)?;

    fields_data.push(field_tokens);
  }

  let schema_impls = create_schema_impls(struct_name, &message_attrs, fields_data);

  output_tokens.extend(schema_impls);

  Ok(output_tokens)
}

#[allow(clippy::collapsible_if)]
pub fn set_map_proto_type(
  mut proto_map: ProtoMap,
  rust_type: &RustType,
) -> Result<ProtoMap, Error> {
  let proto_values = &mut proto_map.values;

  if let ProtoMapValues::Message(path) = proto_values {
    if !matches!(path, MessagePath::Path(_)) {
      let value_path = if let RustType::Map((_, v)) = &rust_type {
        v.clone()
      } else {
        return Err(spanned_error!(
          path,
          "Could not infer the path to the message value, please set it manually"
        ));
      };

      if path.is_suffixed() {
        *path = MessagePath::Path(append_proto_ident(value_path));
      } else {
        *path = MessagePath::Path(value_path);
      }
    }
  } else if let ProtoMapValues::Enum(path) = proto_values {
    if path.is_none() {
      let v = if let RustType::Map((_, v)) = &rust_type {
        v
      } else {
        return Err(spanned_error!(
          path,
          "Could not infer the path to the enum value, please set it manually"
        ));
      };

      *path = Some(v.clone());
    }
  }

  Ok(proto_map)
}
