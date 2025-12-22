use crate::*;

pub fn oneof_schema_impl(
  oneof_attrs: &OneofAttrs,
  enum_ident: &Ident,
  variants_tokens: Vec<TokenStream2>,
  manually_set_tags: &[ManuallySetTag],
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

  let tags = manually_set_tags.iter().map(|m| m.tag);

  quote! {
    impl ::prelude::ProtoOneof for #enum_ident {
      fn name() -> &'static str {
        #proto_name
      }

      fn tags() -> &'static [i32] {
        &[ #(#tags),* ]
      }

      fn proto_schema() -> ::prelude::Oneof {
        Self::proto_schema()
      }

      fn validate(&self, parent_elements: &mut Vec<FieldPathElement>) -> Result<(), Violations> {
        self.validate(parent_elements)
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
