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

  let required_option_tokens = required.then(|| quote! { options.push(oneof_required()); });

  quote! {
    impl ProtoOneof for #enum_ident {
      fn fields() -> Vec<ProtoField> {
        vec![ #(#variants_tokens,)* ]
      }
    }

    impl #enum_ident {
      #[track_caller]
      pub fn to_oneof() -> Oneof {
        let mut options: Vec<ProtoOption> = vec![ #(#options),* ];

        #required_option_tokens

        Oneof {
          name: #proto_name.into(),
          fields: Self::fields(),
          options,
        }
      }
    }
  }
}
