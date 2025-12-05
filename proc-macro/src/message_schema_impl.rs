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
    nested_messages_tokens.extend(quote! { #ident::proto_schema(), });
  }

  for ident in nested_enums {
    nested_enums_tokens.extend(quote! { #ident::proto_schema(), });
  }

  let validator_tokens = if let Some(call) = validator {
    quote! { #call }
  } else {
    quote! { vec![] }
  };

  let options_tokens = tokens_or_default!(options, quote! { vec![] });

  quote! {
    impl ::prelude::AsProtoType for #struct_name {
      fn proto_type() -> ::prelude::ProtoType {
        ::prelude::ProtoType::Message(
          <Self as ::prelude::ProtoMessage>::proto_path()
        )
      }
    }

    impl ::prelude::ProtoMessage for #struct_name {
      fn proto_path() -> ::prelude::ProtoPath {
        ::prelude::ProtoPath {
          name: #full_name,
          file: #file,
          package: #package,
        }
      }

      fn proto_schema() -> ::prelude::Message {
        Self::proto_schema()
      }
    }

    impl #struct_name {
      pub fn proto_schema() -> ::prelude::Message {
        let mut new_msg = ::prelude::Message {
          name: #proto_name,
          full_name: #full_name,
          package: #package,
          file: #file,
          reserved_names: #reserved_names,
          reserved_numbers: vec![ #reserved_numbers ],
          options: #options_tokens,
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
