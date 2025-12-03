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
    schema_feature,
    ..
  } = oneof_attrs;

  let options_tokens = tokens_or_default!(options, quote! { vec![] });
  let required_option_tokens = required.then(|| quote! { options.push(oneof_required()); });
  let schema_feature_tokens = schema_feature
    .as_ref()
    .map(|feat| quote! { #[cfg(feature = #feat)] });

  quote! {
    #schema_feature_tokens
    impl ::prelude::ProtoOneof for #enum_ident {
      fn fields() -> Vec<::prelude::ProtoField> {
        vec![ #(#variants_tokens,)* ]
      }
    }

    #schema_feature_tokens
    impl #enum_ident {
      pub fn to_oneof() -> ::prelude::Oneof {
        let mut options: Vec<::prelude::ProtoOption> = #options_tokens;

        #required_option_tokens

        ::prelude::Oneof {
          name: #proto_name,
          fields: <Self as ::prelude::ProtoOneof>::fields(),
          options,
        }
      }
    }
  }
}
