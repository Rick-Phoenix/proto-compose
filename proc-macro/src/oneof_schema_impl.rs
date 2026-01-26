use crate::{message_schema_impl::field_schema_tokens, *};

impl OneofCtx<'_> {
  pub fn generate_schema_impl(&self) -> TokenStream2 {
    let enum_ident = self.proto_enum_ident();

    let variants_tokens = if self.variants.is_empty() {
      quote! { unimplemented!() }
    } else {
      let tokens = self
        .variants
        .iter()
        .filter_map(|v| v.as_normal())
        .map(|data| field_schema_tokens(data));

      quote! { #(#tokens),* }
    };

    let OneofAttrs {
      options: options_tokens,
      name: proto_name,
      validators,
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
            name: #proto_name.into(),
            fields: vec![ #variants_tokens ],
            options: #options_tokens.into_iter().collect(),
            #[allow(
              clippy::filter_map_identity,
              clippy::iter_on_empty_collections,
              clippy::iter_on_single_items
            )]
            validators: [ #(::prelude::Validator::<#enum_ident>::schema(&#validators)),* ].into_iter().filter_map(|s| s).collect(),
          }
        }
      }
    }
  }
}
