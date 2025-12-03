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
    schema_feature,
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

  let validator_tokens = if let Some(call) = validator {
    quote! { #call }
  } else {
    quote! { vec![] }
  };

  let options_tokens = tokens_or_default!(options, quote! { vec![] });
  let schema_feature_tokens = schema_feature
    .as_ref()
    .map(|feat| quote! { #[cfg(feature = #feat)] });

  quote! {
    #schema_feature_tokens
    impl ::prelude::AsProtoType for #struct_name {
      fn proto_type() -> ::prelude::ProtoType {
        ::prelude::ProtoType::Single(::prelude::TypeInfo {
          name: #full_name,
          path: Some(::prelude::ProtoPath {
            file: #file,
            package: #package
          })
        })
      }
    }

    #schema_feature_tokens
    impl #struct_name {
      pub fn to_message() -> ::prelude::Message {
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
