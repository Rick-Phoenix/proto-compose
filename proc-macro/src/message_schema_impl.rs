use crate::*;

pub fn fallback_message_schema_impl(
  orig_struct_ident: &Ident,
  shadow_struct_ident: Option<&Ident>,
) -> TokenStream2 {
  let proto_struct = shadow_struct_ident.unwrap_or(orig_struct_ident);

  let mut output = quote! {
    impl ::prelude::AsProtoType for #proto_struct {
      fn proto_type() -> ::prelude::ProtoType {
        unimplemented!()
      }
    }

    impl ::prelude::ProtoMessage for #proto_struct {
      const PACKAGE: &str = "";
      const SHORT_NAME: &str = "";

      fn type_url() -> &'static str {
        unimplemented!()
      }

      fn full_name() -> &'static str {
        unimplemented!()
      }

      fn proto_path() -> ::prelude::ProtoPath {
        unimplemented!()
      }

      fn proto_name() -> &'static str {
        unimplemented!()
      }

      fn proto_schema() -> ::prelude::Message {
        unimplemented!()
      }
    }
  };

  if shadow_struct_ident.is_some() {
    output.extend(quote! {
      impl ::prelude::AsProtoType for #orig_struct_ident {
        fn proto_type() -> ::prelude::ProtoType {
          unimplemented!()
        }
      }
    });
  }

  output
}

impl<T: Borrow<FieldData>> MessageCtx<'_, T> {
  pub fn generate_schema_impls(&self) -> TokenStream2 {
    let MessageAttrs {
      reserved_names,
      reserved_numbers,
      options: message_options,
      name: proto_name,
      parent_message,
      extern_path,
      ..
    } = &self.message_attrs;

    let entries_tokens = self.non_ignored_fields.iter().map(|data| {
      let FieldData {
        tag,
        validator,
        options,
        proto_name,
        proto_field,
        ..
      } = data.borrow();

      if let ProtoField::Oneof(OneofInfo { path, required, .. }) = proto_field {
        quote! {
          ::prelude::MessageEntry::Oneof {
            oneof: <#path as ::prelude::ProtoOneof>::proto_schema(),
            required: #required
          }
        }
      } else {
        let field_type_tokens = proto_field.field_proto_type_tokens();

        let validator_schema_tokens = validator
          .as_ref()
          // For default validators (messages only) we skip the schema generation
          .filter(|v| !v.is_fallback)
          .map_or_else(|| quote! { None }, |e| quote! { Some(#e.into_schema()) });

        quote! {
          ::prelude::MessageEntry::Field(
            ::prelude::Field {
              name: #proto_name,
              tag: #tag,
              options: #options,
              type_: #field_type_tokens,
              validator: #validator_schema_tokens,
            }
          )
        }
      }
    });

    let mut output = TokenStream2::new();

    let proto_struct = self.proto_struct_ident();

    let name_method = if let Some(parent) = parent_message {
      quote! {
        static __NAME: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
          format!("{}.{}", #parent::proto_name(), #proto_name)
        });

        &*__NAME
      }
    } else {
      quote! { #proto_name }
    };

    let registry_parent_message = if let Some(parent) = parent_message {
      quote! { Some(|| #parent::proto_name()) }
    } else {
      quote! { None }
    };

    let rust_path_field = if let Some(path) = extern_path {
      quote! { #path.to_string() }
    } else {
      let rust_ident_str = proto_struct.to_string();

      quote! { format!("::{}::{}", __PROTO_FILE.extern_path, #rust_ident_str) }
    };

    output.extend(quote! {
      ::prelude::inventory::submit! {
        ::prelude::RegistryMessage {
          package: __PROTO_FILE.package,
          parent_message: #registry_parent_message,
          message: || <#proto_struct as ::prelude::ProtoMessage>::proto_schema()
        }
      }

      impl ::prelude::AsProtoType for #proto_struct {
        fn proto_type() -> ::prelude::ProtoType {
          ::prelude::ProtoType::Message(
            <Self as ::prelude::ProtoMessage>::proto_path()
          )
        }
      }

      impl ::prelude::ProtoMessage for #proto_struct {
        const PACKAGE: &str = __PROTO_FILE.package;
        const SHORT_NAME: &str = #proto_name;

        fn type_url() -> &'static str {
          static URL: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
            format!("/{}.{}", #proto_struct::PACKAGE, #proto_struct::proto_name())
          });

          &*URL
        }

        fn full_name() -> &'static str {
          static NAME: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
            format!("{}.{}", #proto_struct::PACKAGE, #proto_struct::proto_name())
          });

          &*NAME
        }

        fn proto_path() -> ::prelude::ProtoPath {
          ::prelude::ProtoPath {
            name: Self::proto_name(),
            file: __PROTO_FILE.file,
            package: __PROTO_FILE.package,
          }
        }

        fn proto_name() -> &'static str {
          #name_method
        }

        fn proto_schema() -> ::prelude::Message {
          ::prelude::Message {
            short_name: #proto_name,
            name: Self::proto_name(),
            file: __PROTO_FILE.file,
            package: __PROTO_FILE.package,
            reserved_names: vec![ #(#reserved_names),* ],
            reserved_numbers: vec![ #reserved_numbers ],
            options: #message_options,
            messages: vec![],
            enums: vec![],
            entries: vec![ #(#entries_tokens,)* ],
            cel_rules: #proto_struct::cel_rules().iter().map(|prog| prog.rule.clone()).collect(),
            rust_path: #rust_path_field
          }
        }
      }
    });

    if let Some(shadow_struct_ident) = &self.shadow_struct_ident {
      let orig_struct_ident = &self.orig_struct_ident;

      output.extend(quote! {
        impl ::prelude::AsProtoType for #orig_struct_ident {
          fn proto_type() -> ::prelude::ProtoType {
            <#shadow_struct_ident as ::prelude::AsProtoType>::proto_type()
          }
        }
      });
    }

    output
  }
}
