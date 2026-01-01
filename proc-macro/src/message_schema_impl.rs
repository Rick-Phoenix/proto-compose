use crate::*;

pub fn message_schema_impls<T>(
  orig_struct_ident: &Ident,
  shadow_struct_ident: Option<&Ident>,
  message_attrs: &MessageAttrs,
  fields: &[T],
) -> TokenStream2
where
  T: Borrow<FieldData>,
{
  let MessageAttrs {
    reserved_names,
    reserved_numbers,
    options: message_options,
    name: proto_name,
    parent_message,
    cel_rules: top_level_cel_rules,
    extern_path,
    ..
  } = message_attrs;

  let entries_tokens = fields.iter().map(|data| {
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
          ::prelude::ProtoField {
            name: #proto_name.to_string(),
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

  let name_method = if let Some(parent) = parent_message {
    quote! {
      static __NAME: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
        format!("{}.{}", #parent::name(), #proto_name)
      });

      &*__NAME
    }
  } else {
    quote! { #proto_name }
  };

  let registry_parent_message = if let Some(parent) = parent_message {
    quote! { Some(|| #parent::name()) }
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

  let top_level_cel_rules_tokens = if let Some(programs) = top_level_cel_rules {
    quote! {
      vec![ #(#programs),* ]
    }
  } else {
    quote! { vec![] }
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
      const PACKAGE: &str = __PROTO_FILE.package;
      const SHORT_NAME: &str = #proto_name;

      fn type_url() -> &'static str {
        static URL: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
          format!("/{}.{}", #orig_struct_ident::PACKAGE, #orig_struct_ident::name())
        });

        &*URL
      }

      fn full_name() -> &'static str {
        static NAME: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
          format!("{}.{}", #orig_struct_ident::PACKAGE, #orig_struct_ident::name())
        });

        &*NAME
      }

      fn cel_rules() -> &'static [CelProgram] {
        static PROGRAMS: std::sync::LazyLock<Vec<::prelude::CelProgram>> = std::sync::LazyLock::new(|| {
          #top_level_cel_rules_tokens
        });

        &PROGRAMS
      }

      fn proto_path() -> ::prelude::ProtoPath {
        ::prelude::ProtoPath {
          name: Self::name(),
          file: __PROTO_FILE.file,
          package: __PROTO_FILE.package,
        }
      }

      fn name() -> &'static str {
        #name_method
      }

      fn proto_schema() -> ::prelude::Message {
        let mut new_msg = ::prelude::Message {
          short_name: #proto_name,
          name: Self::name(),
          file: __PROTO_FILE.file,
          package: __PROTO_FILE.package,
          reserved_names: vec![ #(#reserved_names),* ],
          reserved_numbers: vec![ #reserved_numbers ],
          options: #message_options,
          messages: vec![],
          enums: vec![],
          entries: vec![ #(#entries_tokens,)* ],
          cel_rules: Self::cel_rules().iter().map(|prog| prog.rule.clone()).collect(),
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
        const PACKAGE: &str = __PROTO_FILE.package;
        const SHORT_NAME: &str = #proto_name;

        fn type_url() -> &'static str {
          <#orig_struct_ident as ::prelude::ProtoMessage>::type_url()
        }

        fn full_name() -> &'static str {
          <#orig_struct_ident as ::prelude::ProtoMessage>::full_name()
        }

        fn cel_rules() -> &'static [CelProgram] {
          <#orig_struct_ident as ::prelude::ProtoMessage>::cel_rules()
        }

        fn proto_path() -> ::prelude::ProtoPath {
          <#orig_struct_ident as ::prelude::ProtoMessage>::proto_path()
        }

        fn name() -> &'static str {
          #orig_struct_ident::name()
        }

        fn proto_schema() -> ::prelude::Message {
          #orig_struct_ident::proto_schema()
        }

        fn validate(&self) -> Result<(), Violations> {
          self.validate()
        }

        fn nested_validate(&self, ctx: &mut ValidationCtx) {
          self.nested_validate(ctx);
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
