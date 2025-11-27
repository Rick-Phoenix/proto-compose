use quote::format_ident;

use crate::*;

fn clone_struct_without_attrs(item: &ItemStruct) -> ItemStruct {
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

pub(crate) fn process_message_derive_direct(
  item: &mut ItemStruct,
  message_attrs: MessageAttrs,
) -> Result<TokenStream2, Error> {
  let MessageAttrs {
    reserved_names,
    reserved_numbers,
    options,
    name: proto_name,
    nested_messages,
    nested_enums,
    full_name,
    file,
    package,
    ..
  } = message_attrs;

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

    let FieldAttrs {
      tag,
      validator,
      options,
      name,
      kind,
      oneof_tags,
    } = field_attrs;

    let src_field_type = TypeInfo::from_type(&src_field.ty)?;

    if kind.is_oneof() {
      let oneof_path = src_field_type.as_inner_option_path().ok_or(spanned_error!(
        &src_field.ty,
        "Oneofs must be wrapped in Option"
      ))?;

      let oneof_path_str = oneof_path.to_token_stream().to_string();
      let mut oneof_tags_str = String::new();

      for (i, tag) in oneof_tags.iter().enumerate() {
        oneof_tags_str.push_str(&tag.to_string());

        if i != oneof_tags.len() - 1 {
          oneof_tags_str.push_str(", ");
        }
      }

      let oneof_attr: Attribute =
        parse_quote!(#[prost(oneof = #oneof_path_str, tags = #oneof_tags_str)]);

      src_field.attrs.push(oneof_attr);

      fields_data.push(quote! {
        MessageEntry::Oneof(#oneof_path::to_oneof())
      });

      continue;
    }

    let proto_type = match kind {
      ProtoFieldType::Enum(path) => {
        // Handle the errors here and just say it can't be used for a map
        let enum_path = if let Some(path) = path {
          path
        } else {
          src_field_type.rust_type.inner_path().ok_or(spanned_error!(&src_field.ty, "Failed to extract the inner type. Expected a type, or a type wrapped in Option or Vec"))?.clone()
        };

        ProtoType::Enum(enum_path)
      }
      ProtoFieldType::Message(path) => {
        let msg_path = if let MessagePath::Path(path) = path {
          path
        } else {
          let inner_type = src_field_type.rust_type.inner_path().ok_or(spanned_error!(&src_field.ty, "Failed to extract the inner type. Expected a type, or a type wrapped in Option or Vec"))?.clone();

          if path.is_suffixed() {
            append_proto_ident(inner_type)
          } else {
            inner_type
          }
        };

        ProtoType::Message(msg_path)
      }
      ProtoFieldType::Map(proto_map) => {
        ProtoType::Map(set_map_proto_type(proto_map, &src_field_type.rust_type)?)
      }
      // No manually set type, let's try to infer it as a primitive
      // maybe use the larger error for any of these
      _ => match &src_field_type.rust_type {
        RustType::Option(path) => ProtoType::from_primitive(path)?,
        RustType::Boxed(path) => ProtoType::from_primitive(path)?,
        RustType::Vec(path) => ProtoType::from_primitive(path)?,
        RustType::Normal(path) => ProtoType::from_primitive(path)?,
        RustType::Map((k, v)) => {
          let keys = ProtoMapKeys::from_path(k)?;
          let values = ProtoMapValues::from_path(v).map_err(|_| spanned_error!(v, format!("Unrecognized proto map value type {}. If you meant to use an enum or a message, use the attribute", v.to_token_stream())))?;

          let proto_map = ProtoMap { keys, values };

          ProtoType::Map(set_map_proto_type(proto_map, &src_field_type.rust_type)?)
        }
      },
    };

    let prost_attr = ProstAttrs::from_type_info(&src_field_type.rust_type, proto_type.clone(), tag);

    let field_prost_attr: Attribute = parse_quote!(#prost_attr);

    src_field.attrs.push(field_prost_attr);

    // Use new validator but with cardinality info
    let validator_tokens = if let Some(validator) = validator {
      src_field_type.validator_tokens(&validator, &proto_type)
    } else {
      quote! { None }
    };

    // This probably should be the destination proto type just to be sure it's implemented
    // or we change how the trait is expressed in terms of cardinality
    let field_type_tokens = src_field_type.as_proto_type_trait_expr(&proto_type);

    fields_data.push(quote! {
      MessageEntry::Field(
        ProtoField {
          name: #name.to_string(),
          tag: #tag,
          options: #options,
          type_: #field_type_tokens,
          validator: #validator_tokens,
        }
      )
    });
  }

  let mut nested_messages_tokens = TokenStream2::new();
  let mut nested_enums_tokens = TokenStream2::new();

  for ident in nested_messages {
    nested_messages_tokens.extend(quote! { #ident::to_message(), });
  }

  for ident in nested_enums {
    nested_enums_tokens.extend(quote! { #ident::to_enum(), });
  }

  output_tokens.extend(quote! {
    impl ProtoMessage for #struct_name {}

    impl ProtoValidator<#struct_name> for ValidatorMap {
      type Builder = MessageValidatorBuilder;

      fn builder() -> Self::Builder {
        MessageValidator::builder()
      }
    }

    impl AsProtoType for #struct_name {
      fn proto_type() -> ProtoType {
        ProtoType::Single(TypeInfo {
          name: #full_name,
          path: Some(ProtoPath {
            file: #file.into(),
            package: #package.into()
          })
        })
      }
    }

    impl #struct_name {
      #[track_caller]
      pub fn to_message() -> Message {
        let mut new_msg = Message {
          name: #proto_name,
          full_name: #full_name,
          package: #package.into(),
          file: #file.into(),
          reserved_names: #reserved_names,
          reserved_numbers: vec![ #reserved_numbers ],
          options: #options,
          messages: vec![ #nested_messages_tokens ],
          enums: vec![ #nested_enums_tokens ],
          entries: vec![ #(#fields_data,)* ],
        };

        new_msg
      }
    }
  });

  Ok(output_tokens)
}

pub(crate) fn process_message_derive_shadow(
  item: &mut ItemStruct,
  message_attrs: MessageAttrs,
) -> Result<TokenStream2, Error> {
  let MessageAttrs {
    reserved_names,
    reserved_numbers,
    options,
    name: proto_name,
    nested_messages,
    nested_enums,
    full_name,
    file,
    package,
    ..
  } = message_attrs;

  let mut output_tokens = TokenStream2::new();

  let mut shadow_struct = clone_struct_without_attrs(item);

  let ItemStruct {
    ident: orig_struct_name,
    fields,
    ..
  } = item;

  let mut fields_data: Vec<TokenStream2> = Vec::new();

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

    let FieldAttrs {
      tag,
      validator,
      options,
      name,
      kind,
      oneof_tags,
    } = field_attrs;

    let src_field_type = TypeInfo::from_type(&src_field.ty)?;

    if kind.is_oneof() {
      let oneof_path = src_field_type.as_inner_option_path().ok_or(spanned_error!(
        &src_field.ty,
        "Oneofs must be wrapped in Option"
      ))?;

      let oneof_path_str = oneof_path.to_token_stream().to_string();
      let mut oneof_tags_str = String::new();

      for (i, tag) in oneof_tags.iter().enumerate() {
        oneof_tags_str.push_str(&tag.to_string());

        if i != oneof_tags.len() - 1 {
          oneof_tags_str.push_str(", ");
        }
      }

      let oneof_attr: Attribute =
        parse_quote!(#[prost(oneof = #oneof_path_str, tags = #oneof_tags_str)]);

      dst_field.attrs.push(oneof_attr);

      fields_data.push(quote! {
        MessageEntry::Oneof(#oneof_path::to_oneof())
      });

      continue;
    }

    let proto_type = match kind {
      ProtoFieldType::Enum(path) => {
        // Handle the errors here and just say it can't be used for a map
        let enum_path = if let Some(path) = path {
          path
        } else {
          src_field_type.rust_type.inner_path().ok_or(spanned_error!(&src_field.ty, "Failed to extract the inner type. Expected a type, or a type wrapped in Option or Vec"))?.clone()
        };

        ProtoType::Enum(enum_path)
      }
      ProtoFieldType::Message(path) => {
        let msg_path = if let MessagePath::Path(path) = path {
          path
        } else {
          let inner_type = src_field_type.rust_type.inner_path().ok_or(spanned_error!(&src_field.ty, "Failed to extract the inner type. Expected a type, or a type wrapped in Option or Vec"))?.clone();

          if path.is_suffixed() {
            append_proto_ident(inner_type)
          } else {
            inner_type
          }
        };

        ProtoType::Message(msg_path)
      }
      ProtoFieldType::Map(proto_map) => {
        ProtoType::Map(set_map_proto_type(proto_map, &src_field_type.rust_type)?)
      }
      // No manually set type, let's try to infer it as a primitive
      // maybe use the larger error for any of these
      _ => match &src_field_type.rust_type {
        RustType::Option(path) => ProtoType::from_primitive(path)?,
        RustType::Boxed(path) => ProtoType::from_primitive(path)?,
        RustType::Vec(path) => ProtoType::from_primitive(path)?,
        RustType::Normal(path) => ProtoType::from_primitive(path)?,
        RustType::Map((k, v)) => {
          let keys = ProtoMapKeys::from_path(k)?;
          let values = ProtoMapValues::from_path(v).map_err(|_| spanned_error!(v, format!("Unrecognized proto map value type {}. If you meant to use an enum or a message, use the attribute", v.to_token_stream())))?;

          let proto_map = ProtoMap { keys, values };

          ProtoType::Map(set_map_proto_type(proto_map, &src_field_type.rust_type)?)
        }
      },
    };

    let proto_output_type_inner = proto_type.output_proto_type();

    // Get output type
    let proto_output_type_outer: Type = match &src_field_type.rust_type {
      RustType::Option(_) => parse_quote! { Option<#proto_output_type_inner> },
      RustType::Boxed(_) => parse_quote! { Option<Box<#proto_output_type_inner>> },
      RustType::Map(_) => parse_quote!( #proto_output_type_inner ),
      RustType::Vec(_) => parse_quote! { Vec<#proto_output_type_inner> },
      RustType::Normal(_) => parse_quote!( #proto_output_type_inner ),
    };

    dst_field.ty = proto_output_type_outer;

    let prost_attr = ProstAttrs::from_type_info(&src_field_type.rust_type, proto_type.clone(), tag);

    let field_prost_attr: Attribute = parse_quote!(#prost_attr);

    dst_field.attrs.push(field_prost_attr);

    // Use new validator but with cardinality info
    let validator_tokens = if let Some(validator) = validator {
      src_field_type.validator_tokens(&validator, &proto_type)
    } else {
      quote! { None }
    };

    // This probably should be the destination proto type just to be sure it's implemented
    // or we change how the trait is expressed in terms of cardinality
    let field_type_tokens = src_field_type.as_proto_type_trait_expr(&proto_type);

    fields_data.push(quote! {
      MessageEntry::Field(
        ProtoField {
          name: #name.to_string(),
          tag: #tag,
          options: #options,
          type_: #field_type_tokens,
          validator: #validator_tokens,
        }
      )
    });
  }

  let mut nested_messages_tokens = TokenStream2::new();
  let mut nested_enums_tokens = TokenStream2::new();

  for ident in nested_messages {
    nested_messages_tokens.extend(quote! { #ident::to_message(), });
  }

  for ident in nested_enums {
    nested_enums_tokens.extend(quote! { #ident::to_enum(), });
  }

  output_tokens.extend(quote! {
    impl ProtoMessage for #orig_struct_name {}

    impl ProtoValidator<#orig_struct_name> for ValidatorMap {
      type Builder = MessageValidatorBuilder;

      fn builder() -> Self::Builder {
        MessageValidator::builder()
      }
    }

    impl AsProtoType for #orig_struct_name {
      fn proto_type() -> ProtoType {
        ProtoType::Single(TypeInfo {
          name: #full_name,
          path: Some(ProtoPath {
            file: #file.into(),
            package: #package.into()
          })
        })
      }
    }

    impl #orig_struct_name {
      #[track_caller]
      pub fn to_message() -> Message {
        let mut new_msg = Message {
          name: #proto_name,
          full_name: #full_name,
          package: #package.into(),
          file: #file.into(),
          reserved_names: #reserved_names,
          reserved_numbers: vec![ #reserved_numbers ],
          options: #options,
          messages: vec![ #nested_messages_tokens ],
          enums: vec![ #nested_enums_tokens ],
          entries: vec![ #(#fields_data,)* ],
        };

        new_msg
      }
    }
  });

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
