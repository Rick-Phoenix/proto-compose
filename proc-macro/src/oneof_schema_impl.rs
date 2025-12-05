use crate::*;

pub fn oneof_schema_impl(
  oneof_attrs: &OneofAttrs,
  enum_ident: &Ident,
  variants_tokens: Vec<TokenStream2>,
) -> TokenStream2 {
  let OneofAttrs {
    options,
    name: proto_name,
    required,
    ..
  } = oneof_attrs;

  let options_tokens = tokens_or_default!(options, quote! { vec![] });
  let required_option_tokens =
    required.then(|| quote! { options.push(::prelude::oneof_required()); });

  quote! {
    impl ::prelude::ProtoOneof for #enum_ident {
      fn proto_schema() -> ::prelude::Oneof {
        Self::proto_schema()
      }
    }

    impl #enum_ident {
      pub fn proto_schema() -> ::prelude::Oneof {
        let mut options: Vec<::prelude::ProtoOption> = #options_tokens;

        #required_option_tokens

        ::prelude::Oneof {
          name: #proto_name,
          fields: vec![ #(#variants_tokens,)* ],
          options,
        }
      }
    }
  }
}
