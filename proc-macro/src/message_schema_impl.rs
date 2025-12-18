use crate::*;

pub struct MessageSchemaImplsCtx<'a> {
  pub orig_struct_ident: &'a Ident,
  pub shadow_struct_ident: Option<&'a Ident>,
  pub message_attrs: &'a MessageAttrs,
  pub entries_tokens: Vec<TokenStream2>,
  pub top_level_programs_ident: Option<&'a Ident>,
}

pub fn message_schema_impls(ctx: MessageSchemaImplsCtx) -> TokenStream2 {
  let MessageSchemaImplsCtx {
    orig_struct_ident,
    shadow_struct_ident,
    message_attrs:
      MessageAttrs {
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
      },
    entries_tokens,
    top_level_programs_ident,
  } = ctx;

  let mut output = TokenStream2::new();

  let mut nested_messages_tokens = TokenStream2::new();
  let mut nested_enums_tokens = TokenStream2::new();

  for ident in nested_messages {
    nested_messages_tokens.extend(quote! { #ident::proto_schema(), });
  }

  for ident in nested_enums {
    nested_enums_tokens.extend(quote! { #ident::proto_schema(), });
  }

  let options_tokens = tokens_or_default!(options, quote! { vec![] });

  let cel_rules_field = top_level_programs_ident.map_or_else(
    || quote! { vec![] },
    |ident| {
      quote! {
        #ident.iter().map(|prog| &prog.rule).collect()
      }
    },
  );

  output.extend(quote! {
    impl ::prelude::AsProtoType for #orig_struct_ident {
      fn proto_type() -> ::prelude::ProtoType {
        ::prelude::ProtoType::Message(
          <Self as ::prelude::ProtoMessage>::proto_path()
        )
      }
    }

    impl ::prelude::ProtoMessage for #orig_struct_ident {
      fn proto_path() -> ::prelude::ProtoPath {
        ::prelude::ProtoPath {
          name: #full_name,
          file: #file,
          package: #package,
        }
      }

      fn proto_schema() -> ::prelude::Message {
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
  });

  if let Some(shadow_struct_ident) = shadow_struct_ident {
    output.extend(quote! {
      impl ::prelude::ProtoMessage for #shadow_struct_ident {
        fn proto_path() -> ::prelude::ProtoPath {
          <#orig_struct_ident as ::prelude::ProtoMessage>::proto_path()
        }

        fn proto_schema() -> ::prelude::Message {
          #orig_struct_ident::proto_schema()
        }

        fn validate(&self) -> Result<(), Violations> {
          self.validate()
        }

        fn nested_validate(&self, field_context: &FieldContext, parent_elements: &mut Vec<FieldPathElement>) -> Result<(), Violations> {
          self.nested_validate(field_context, parent_elements)
        }
      }

      impl ::prelude::AsProtoType for #shadow_struct_ident {
        fn proto_type() -> ::prelude::ProtoType {
          <#orig_struct_ident as ::prelude::AsProtoType>::proto_type()
        }
      }
    });
  }

  output
}
