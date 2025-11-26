use std::{borrow::Cow, str::FromStr};

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
    ident: item.ident.clone(),
    generics: item.generics.clone(),
    fields: Fields::Named(syn::FieldsNamed {
      brace_token: token::Brace::default(),
      named: item_fields.collect(),
    }),
    semi_token: None,
  }
}

pub(crate) fn process_message_derive(item: &mut ItemStruct) -> Result<TokenStream2, Error> {
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
    into,
  } = process_derive_message_attrs(&item.ident, &item.attrs)?;

  let make_shadow_struct = into;

  let shadow_struct = if make_shadow_struct {
    Some(clone_struct_without_attrs(item))
  } else {
    None
  };

  let ItemStruct {
    ident: struct_name,
    fields,
    ..
  } = item;

  let mut fields_data: Vec<TokenStream2> = Vec::new();

  for field in fields {
    let field_name = field.ident.as_ref().expect("Expected named field");

    let field_attrs = if let Some(attrs) = process_derive_field_attrs(field_name, &field.attrs)? {
      attrs
    } else {
      continue;
    };

    let FieldAttrs {
      tag,
      validator,
      options,
      name,
      custom_type,
      kind,
      oneof_tags,
    } = field_attrs;

    let field_type = TypeInfo::from_type(&field.ty)?;

    if kind.is_oneof() {
      let oneof_path = field_type.as_inner_option_path().ok_or(spanned_error!(
        &field.ty,
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
        parse_quote!(#[proto(oneof = #oneof_path_str, tags = #oneof_tags_str)]);

      field.attrs.push(oneof_attr);

      fields_data.push(quote! {
        MessageEntry::Oneof(#oneof_path::to_oneof())
      });

      continue;
    }

    let proto_type = if let Some(custom_type) = &custom_type {
      let path_wrapper = PathWrapper::new(Cow::Borrowed(custom_type));

      let last_segment = path_wrapper.last_segment();

      let last_segment_str = last_segment.ident().to_string();

      match last_segment_str.as_str() {
        "ProtoEnum" => {
          let path = field_type
            .as_inner_option_path()
            .unwrap_or(field_type.full_type.as_ref());

          ProtoType::Enum(path.clone())
        }
        "ProtoMessage" => ProtoType::Message,
        _ => ProtoType::from_rust_type(&field_type.rust_type)?,
      }
    } else if let RustType::Map((k, v)) = &field_type.rust_type {
      let keys_str = k.require_ident()?.to_string();
      let values_str = v.require_ident()?.to_string();

      let keys = ProtoMapKeys::from_str(&keys_str).unwrap();
      let values = if values_str == "ProtoEnum" {
        ProtoMapValues::Enum(v.clone())
      } else {
        ProtoMapValues::from_str(&values_str).map_err(|e| spanned_error!(&field.ty, e))?
      };

      ProtoType::Map(Box::new(ProtoMap { keys, values }))
    } else {
      ProtoType::from_rust_type(&field_type.rust_type)?
    };

    let prost_attr = ProstAttrs::from_type_info(&field_type.rust_type, proto_type.clone(), tag);

    let field_prost_attr: Attribute = parse_quote!(#prost_attr);

    field.attrs.push(field_prost_attr);

    let validator_tokens = if let Some(validator) = validator {
      field_type.validator_tokens(&validator)
    } else {
      quote! { None }
    };

    let full_type_path = &field_type.full_type;

    let field_type_tokens = quote! { <#full_type_path as AsProtoType>::proto_type() };

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

  let output = quote! {
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
  };

  Ok(output)
}
