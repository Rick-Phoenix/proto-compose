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
    } = field_attrs;

    let mut field_type = match &field.ty {
      Type::Path(type_path) => &type_path.path,

      _ => panic!("Must be a path type"),
    };

    let processed_type = extract_type_from_path(field_type);

    let proto_type = processed_type.path();

    if is_oneof {
      fields_data.push(quote! {
        MessageEntry::Oneof(#proto_type::to_oneof())
      });

      continue;
    }

    let is_optional = processed_type.is_option();

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

    let field_type_tokens = if is_optional {
      quote! { <Option<#proto_type> as AsProtoType>::proto_type() }
    } else {
      quote! { <#proto_type as AsProtoType>::proto_type() }
    };

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
          // messages: vec![ #nested_messages ],
          // enums: vec![ #nested_enums ],
          entries: vec![ #(#fields_data,)* ],
          ..Default::default()
        };

        new_msg
      }
    }
  };

  Ok(output)
}
