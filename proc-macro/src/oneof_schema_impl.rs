use crate::*;

pub fn oneof_schema_impl<T>(
  oneof_attrs: &OneofAttrs,
  enum_ident: &Ident,
  variants: &[T],
  manually_set_tags: &[ManuallySetTag],
) -> TokenStream2
where
  T: Borrow<FieldData>,
{
  let variants_tokens = variants.iter().map(|data| {
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
      ::prelude::ProtoField {
        name: #proto_name.to_string(),
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
  } = oneof_attrs;

  quote! {
    impl ::prelude::ProtoOneof for #enum_ident {
      const NAME: &str = #proto_name;
      const TAGS: &[i32] = &[ #(#manually_set_tags),* ];

      fn proto_schema() -> ::prelude::Oneof {
        Self::proto_schema()
      }
    }

    impl #enum_ident {
      pub fn proto_schema() -> ::prelude::Oneof {
        ::prelude::Oneof {
          name: #proto_name,
          fields: vec![ #(#variants_tokens,)* ],
          options: #options_tokens,
        }
      }
    }
  }
}
