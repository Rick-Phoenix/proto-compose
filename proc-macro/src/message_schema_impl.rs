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
        parent_message,
        extern_path,
        ..
      },
    entries_tokens,
    top_level_programs_ident,
  } = ctx;

  let mut output = TokenStream2::new();

  let options_tokens = tokens_or_default!(options, quote! { vec![] });

  let cel_rules_field = top_level_programs_ident.map_or_else(
    || quote! { vec![] },
    |ident| {
      quote! {
        #ident.iter().map(|prog| &prog.rule).collect()
      }
    },
  );

  let full_name_method = if let Some(parent) = parent_message {
    quote! {
      static __FULL_NAME: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
        format!("{}.{}", #parent::full_name(), #proto_name).into()
      });

      &*__FULL_NAME
    }
  } else {
    quote! { #proto_name }
  };

  let registry_parent_message = if let Some(parent) = parent_message {
    quote! { Some(|| #parent::full_name()) }
  } else {
    quote! { None }
  };

  let rust_path_field = if let Some(path) = extern_path {
    quote! { #path.to_string() }
  } else {
    let rust_ident_str =
      shadow_struct_ident.map_or_else(|| orig_struct_ident.to_string(), |id| id.to_string());

    quote! { format!("::{}::{}", __PROTO_FILE.extern_path, #rust_ident_str) }
  };

  output.extend(quote! {
    ::prelude::inventory::submit! {
      ::prelude::RegistryMessage {
        package: __PROTO_FILE.package,
        parent_message: #registry_parent_message,
        message: || #orig_struct_ident::proto_schema()
      }
    }

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
          name: Self::full_name(),
          file: __PROTO_FILE.file,
          package: __PROTO_FILE.package,
        }
      }

      fn full_name() -> &'static str {
        #full_name_method
      }

      fn proto_schema() -> ::prelude::Message {
        let mut new_msg = ::prelude::Message {
          name: #proto_name,
          full_name: Self::full_name(),
          file: __PROTO_FILE.file,
          package: __PROTO_FILE.package,
          reserved_names: vec![ #(#reserved_names),* ],
          reserved_numbers: vec![ #reserved_numbers ],
          options: #options_tokens,
          messages: vec![],
          enums: vec![],
          entries: vec![ #(#entries_tokens,)* ],
          cel_rules: #cel_rules_field,
          rust_path: #rust_path_field
        };

        new_msg
      }
    }
  });

  if let Some(shadow_struct_ident) = shadow_struct_ident {
    output.extend(quote! {
      #[allow(clippy::ptr_arg)]
      impl ::prelude::ProtoMessage for #shadow_struct_ident {
        fn proto_path() -> ::prelude::ProtoPath {
          <#orig_struct_ident as ::prelude::ProtoMessage>::proto_path()
        }

        fn full_name() -> &'static str {
          #orig_struct_ident::full_name()
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
