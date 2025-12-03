use crate::*;

pub fn message_schema_impls(
  struct_name: &Ident,
  message_attrs: &MessageAttrs,
  fields_data: Vec<TokenStream2>,
) -> TokenStream2 {
  let MessageAttrs {
    reserved_names,
    reserved_numbers,
    options,
    name: proto_name,
    full_name,
    file,
    package,
    nested_messages,
    nested_enums,
    validator,
    ..
  } = message_attrs;

  let mut nested_messages_tokens = TokenStream2::new();
  let mut nested_enums_tokens = TokenStream2::new();

  for ident in nested_messages {
    nested_messages_tokens.extend(quote! { #ident::to_message(), });
  }

  for ident in nested_enums {
    nested_enums_tokens.extend(quote! { #ident::to_enum(), });
  }

  let validator_tokens = if let Some(validator) = validator {
    quote! { #validator }
  } else {
    quote! { vec![] }
  };

  quote! {
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
            file: #file,
            package: #package
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
          package: #package,
          file: #file,
          reserved_names: #reserved_names,
          reserved_numbers: vec![ #reserved_numbers ],
          options: vec![ #(#options),* ],
          messages: vec![ #nested_messages_tokens ],
          enums: vec![ #nested_enums_tokens ],
          entries: vec![ #(#fields_data,)* ],
          cel_rules: #validator_tokens,
        };

        new_msg
      }
    }
  }
}
