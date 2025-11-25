use crate::*;

pub(crate) fn process_message_derive(item: &mut ItemStruct) -> Result<TokenStream2, Error> {
  let ItemStruct {
    attrs,
    ident: struct_name,
    fields,
    ..
  } = item;

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
  } = process_derive_message_attrs(struct_name, attrs)?;

  let make_shadow_struct = into;

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

    let field_type_path = extract_type_path(&field.ty)?;

    let field_type = extract_type(&field.ty)?;

    let proto_type = field_type.inner();

    if kind.is_oneof() {
      if !field_type.is_option() {
        return Err(spanned_error!(
          &field.ty,
          "Oneofs must be wrapped in Option"
        ));
      }

      let oneof_path_str = proto_type.to_token_stream().to_string();
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
        MessageEntry::Oneof(#proto_type::to_oneof())
      });

      continue;
    }

    let proto_type2 = get_proto_type_outer(field_type_path);
    let tag_as_str = tag.to_string();

    let field_prost_attr: Attribute = parse_quote!(#[proto(#proto_type2, tag2 = #tag_as_str)]);

    field.attrs.push(field_prost_attr);

    let validator_tokens = if let Some(validator) = validator {
      match validator {
        ValidatorExpr::Call(call) => {
          quote! { Some(<ValidatorMap as ProtoValidator<#proto_type>>::from_builder(#call)) }
        }
        ValidatorExpr::Closure(closure) => {
          quote! { Some(<ValidatorMap as ProtoValidator<#proto_type>>::build_rules(#closure)) }
        }
      }
    } else {
      quote! { None }
    };

    let field_type_tokens = quote! { <#field_type_path as AsProtoType>::proto_type() };

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
