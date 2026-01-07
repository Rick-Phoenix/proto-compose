use crate::*;

pub fn fallback_oneof_schema_impl(enum_ident: &Ident) -> TokenStream2 {
  quote! {
    impl ::prelude::ProtoOneof for #enum_ident {
      const NAME: &str = "";
      const TAGS: &[i32] = &[];

      fn proto_schema() -> ::prelude::Oneof {
        unimplemented!()
      }
    }
  }
}

impl<T: Borrow<FieldData>> OneofCtx<'_, T> {
  pub fn generate_schema_impl(&self) -> TokenStream2 {
    let enum_ident = self.proto_enum_ident();

    let variants_tokens = self.non_ignored_variants.iter().map(|data| {
      let FieldData {
        tag,
        validator,
        options,
        proto_name,
        proto_field,
        ..
      } = data.borrow();

      let field_type_tokens = proto_field.field_proto_type_tokens();

      let validator_schema_tokens = validator
        .as_ref()
        // For default validators (messages only) we skip the schema generation
        .filter(|v| !v.is_fallback)
        .map_or_else(|| quote! { None }, |e| quote! { Some(#e.into_schema()) });

      quote! {
        ::prelude::Field {
          name: #proto_name,
          tag: #tag,
          options: #options,
          type_: #field_type_tokens,
          validator: #validator_schema_tokens,
        }
      }
    });

    let OneofAttrs {
      options: options_tokens,
      name: proto_name,
      ..
    } = &self.oneof_attrs;
    let tags = &self.tags;

    quote! {
      impl ::prelude::ProtoOneof for #enum_ident {
        #[doc(hidden)]
        const NAME: &str = #proto_name;
        #[doc(hidden)]
        const TAGS: &[i32] = &[ #(#tags),* ];

        fn proto_schema() -> ::prelude::Oneof {
          ::prelude::Oneof {
            name: #proto_name,
            fields: vec![ #(#variants_tokens,)* ],
            options: #options_tokens,
          }
        }
      }
    }
  }
}
