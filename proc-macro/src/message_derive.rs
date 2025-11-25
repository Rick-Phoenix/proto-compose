use crate::*;

pub(crate) fn process_message_derive(tokens: DeriveInput) -> Result<TokenStream2, Error> {
  let DeriveInput {
    attrs,
    ident: struct_name,
    data,
    ..
  } = tokens;

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
  } = process_derive_message_attrs(&struct_name, &attrs).unwrap();

  let data = if let Data::Struct(struct_data) = data {
    struct_data
  } else {
    panic!()
  };

  let fields = if let Fields::Named(fields) = data.fields {
    fields.named
  } else {
    panic!()
  };

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
      is_oneof,
      custom_type,
    } = field_attrs;

    if reserved_numbers.contains(tag) {
      return Err(spanned_error!(
        field,
        format!("Tag number {tag} is reserved.")
      ));
    }

    let field_type = extract_type(&field.ty)?;

    let proto_type = field_type.inner();
    let outer_type = &field_type.outer;

    if is_oneof {
      if !field_type.is_option() {
        return Err(spanned_error!(
          &field.ty,
          "Oneofs must be wrapped in Option"
        ));
      }

      fields_data.push(quote! {
        MessageEntry::Oneof(#proto_type::to_oneof())
      });

      continue;
    }

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

    let field_type_tokens = quote! { <#outer_type as AsProtoType>::proto_type() };

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
