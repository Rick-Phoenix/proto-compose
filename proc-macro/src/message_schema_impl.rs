use crate::*;

pub fn message_schema_impls(
  struct_name: &Ident,
  message_attrs: &MessageAttrs,
  entries_tokens: Vec<TokenStream2>,
  fields_cel_rules: Vec<TokenStream2>,
  top_level_programs_ident: Option<&Ident>,
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

  let options_tokens = tokens_or_default!(options, quote! { vec![] });

  let cel_rules_method = top_level_programs_ident.map(|ident| {
    quote! {
      fn cel_rules() -> Vec<&'static CelRule> {
        use ::prelude::{ProtoValidator, Validator, ValidationResult, field_context::Violations};

        let mut rules_agg: Vec<&CelRule> = #ident.iter().map(|prog| &prog.rule).collect();

        #(
          rules_agg.extend(#fields_cel_rules);
        )*

        rules_agg
      }
    }
  });

  let cel_rules_field = top_level_programs_ident.map_or_else(
    || quote! { vec![] },
    |ident| {
      quote! {
        #ident.iter().map(|prog| &prog.rule).collect()
      }
    },
  );

  quote! {
    impl ::prelude::AsProtoType for #struct_name {
      fn proto_type() -> ::prelude::ProtoType {
        ::prelude::ProtoType::Message(
          <Self as ::prelude::ProtoMessage>::proto_path()
        )
      }
    }

    impl ::prelude::ProtoMessage for #struct_name {
      #cel_rules_method

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
          reserved_names: vec![ #(#reserved_names),* ],
          reserved_numbers: vec![ #reserved_numbers ],
          options: #options_tokens,
          messages: vec![ #nested_messages_tokens ],
          enums: vec![ #nested_enums_tokens ],
          entries: vec![ #(#entries_tokens,)* ],
          cel_rules: #cel_rules_field,
        };

        new_msg
      }
    }
  }
}
