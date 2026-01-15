use crate::*;

pub fn field_schema_tokens(data: &FieldData) -> TokenStream2 {
  let FieldData {
    tag,
    validator,
    options,
    proto_name,
    proto_field,
    deprecated,
    span,
    ..
  } = data;

  if let ProtoField::Oneof(OneofInfo { path, required, .. }) = proto_field {
    if options.is_default() {
      quote_spanned! {*span=>
        ::prelude::MessageEntry::Oneof {
          oneof: <#path as ::prelude::ProtoOneof>::proto_schema(),
          required: #required
        }
      }
    } else {
      quote_spanned! {*span=>
        ::prelude::MessageEntry::Oneof {
          oneof: <#path as ::prelude::ProtoOneof>::proto_schema().with_options(#options),
          required: #required
        }
      }
    }
  } else {
    let field_type_tokens = proto_field.proto_field_target_type(*span);

    let validator_schema_tokens = validator
      .as_ref()
      // For default validators (messages only) we skip the schema generation
      .filter(|v| !v.is_fallback)
      .map_or_else(
        || quote_spanned! {*span=> None },
        |e| {
          if let ProtoField::Map(_) = proto_field {
            let validator_name = data.validator_name();
            let validator_target_type = proto_field.validator_target_type(*span);

            quote_spanned! {*span=>
              Some(<#validator_name as ::prelude::Validator<#validator_target_type>>::into_schema(#e))
            }
          } else {
            quote_spanned! {*span=> Some(#e.into_schema()) }
          }
        },
      );

    let options_tokens = options_tokens(*span, options, *deprecated);

    quote_spanned! {*span=>
      ::prelude::Field {
        name: #proto_name,
        tag: #tag,
        options: #options_tokens.into_iter().collect(),
        type_: #field_type_tokens,
        validator: #validator_schema_tokens,
      }
    }
  }
}

impl MessageCtx<'_> {
  pub fn generate_schema_impls(&self) -> TokenStream2 {
    let MessageAttrs {
      reserved_names,
      reserved_numbers,
      options: message_options,
      name: proto_name,
      parent_message,
      extern_path,
      deprecated,
      ..
    } = &self.message_attrs;

    let entries_tokens = if self.fields_data.is_empty() {
      quote! { unimplemented!() }
    } else {
      let tokens = self
        .fields_data
        .iter()
        .filter_map(|d| d.as_normal())
        .map(|data| {
          let field = field_schema_tokens(data);

          if data.proto_field.is_oneof() {
            field
          } else {
            quote_spanned! {data.span=>
              ::prelude::MessageEntry::Field(
                #field
              )
            }
          }
        });

      quote! { #(#tokens),* }
    };

    let mut output = TokenStream2::new();

    let proto_struct = self.proto_struct_ident();

    let name_method = if let Some(parent) = parent_message {
      quote_spanned! {parent.span()=>
        static __NAME: ::prelude::Lazy<String> = ::prelude::Lazy::new(|| {
          format!("{}.{}", <#parent as ::prelude::ProtoMessage>::proto_name(), #proto_name)
        });

        &*__NAME
      }
    } else {
      quote! { #proto_name }
    };

    let registry_parent_message = if let Some(parent) = parent_message {
      quote_spanned! {parent.span()=> Some(|| <#parent as ::prelude::ProtoMessage>::proto_name()) }
    } else {
      quote! { None }
    };

    let rust_path_field = if let Some(path) = extern_path {
      quote_spanned! {path.span()=> #path.to_string() }
    } else {
      let rust_ident_str = proto_struct.to_string();

      quote! { format!("::{}::{}", __PROTO_FILE.extern_path, #rust_ident_str) }
    };

    let options_tokens = options_tokens(Span::call_site(), message_options, *deprecated);

    let inventory_call = has_inventory_feat().then(|| {
      quote! {
        ::prelude::inventory::submit! {
          ::prelude::RegistryMessage {
            package: __PROTO_FILE.package,
            parent_message: #registry_parent_message,
            message: || <#proto_struct as ::prelude::ProtoMessage>::proto_schema()
          }
        }
      }
    });

    output.extend(quote! {
      #inventory_call

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
          static URL: ::prelude::Lazy<String> = ::prelude::Lazy::new(|| {
            format!("/{}.{}", <#proto_struct as ::prelude::ProtoMessage>::PACKAGE, <#proto_struct as ::prelude::ProtoMessage>::proto_name())
          });

          &*URL
        }

        fn full_name() -> &'static str {
          static NAME: ::prelude::Lazy<String> = ::prelude::Lazy::new(|| {
            format!("{}.{}", <#proto_struct as ::prelude::ProtoMessage>::PACKAGE, <#proto_struct as ::prelude::ProtoMessage>::proto_name())
          });

          &*NAME
        }

        fn proto_path() -> ::prelude::ProtoPath {
          ::prelude::ProtoPath {
            name: <Self as ::prelude::ProtoMessage>::proto_name(),
            file: __PROTO_FILE.name,
            package: __PROTO_FILE.package,
          }
        }

        fn proto_name() -> &'static str {
          #name_method
        }

        fn proto_schema() -> ::prelude::Message {
          ::prelude::Message {
            short_name: #proto_name,
            name: <Self as ::prelude::ProtoMessage>::proto_name(),
            file: __PROTO_FILE.name,
            package: __PROTO_FILE.package,
            reserved_names: vec![ #(#reserved_names),* ],
            reserved_numbers: #reserved_numbers,
            options: #options_tokens.into_iter().collect(),
            messages: vec![],
            enums: vec![],
            entries: vec![ #entries_tokens ],
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
