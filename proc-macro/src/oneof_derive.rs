use crate::*;

fn create_shadow_enum(item: &ItemEnum) -> ItemEnum {
  let variants = item.variants.iter().map(|variant| Variant {
    attrs: vec![],
    ident: variant.ident.clone(),
    discriminant: variant.discriminant.clone(),
    fields: variant.fields.clone(),
  });

  ItemEnum {
    attrs: vec![],
    vis: Visibility::Public(token::Pub::default()),
    enum_token: token::Enum::default(),
    ident: format_ident!("{}Proto", item.ident),
    generics: item.generics.clone(),
    brace_token: token::Brace::default(),
    variants: variants.collect(),
  }
}

pub fn process_oneof_derive(item: &mut ItemEnum) -> Result<TokenStream2, Error> {
  let oneof_attrs = process_oneof_attrs(&item.ident, &item.attrs, false)?;

  if oneof_attrs.direct {
    process_oneof_derive_direct(item, oneof_attrs)
  } else {
    todo!()
  }
}

pub(crate) fn process_oneof_derive_direct(
  item: &mut ItemEnum,
  oneof_attrs: OneofAttrs,
) -> Result<TokenStream2, Error> {
  let ItemEnum {
    attrs,
    ident: enum_name,
    variants,
    ..
  } = item;

  let OneofAttrs {
    options,
    name: proto_name,
    required,
    ..
  } = oneof_attrs;

  let prost_derive: Attribute = parse_quote!(#[derive(prost::Oneof, PartialEq, Clone)]);

  attrs.push(prost_derive);

  let mut variants_tokens: Vec<TokenStream2> = Vec::new();

  for variant in variants {
    let field_attrs = process_derive_field_attrs(&variant.ident, &variant.attrs)?;

    let FieldAttrs {
      tag,
      validator,
      options,
      name,
      kind,
      ..
    } = field_attrs;

    let variant_type = if let Fields::Unnamed(variant_fields) = &variant.fields {
      if variant_fields.unnamed.len() != 1 {
        panic!("Oneof variants must contain a single value");
      }

      variant_fields.unnamed.first().unwrap().ty.clone()
    } else {
      panic!("Enum can only have one unnamed field")
    };

    let type_info = TypeInfo::from_type(&variant_type, kind)?;

    let proto_type = &type_info.proto_type;

    let prost_attr_tokens =
      ProstAttrs::from_type_info(&type_info.rust_type, proto_type.clone(), tag);

    let prost_attr: Attribute = parse_quote!(#prost_attr_tokens);

    variant.attrs.push(prost_attr);

    let validator_tokens = if let Some(validator) = validator {
      type_info.validator_tokens(&validator, &proto_type)
    } else {
      quote! { None }
    };

    let field_type_tokens = type_info.as_proto_type_trait_expr(&proto_type);

    variants_tokens.push(quote! {
      ProtoField {
        name: #name.to_string(),
        options: #options,
        type_: #field_type_tokens,
        validator: #validator_tokens,
        tag: #tag,
      }
    });
  }

  let required_option_tokens = required.then(|| quote! { options.push(oneof_required()); });

  let output_tokens = quote! {
    impl ProtoOneof for #enum_name {
      fn fields() -> Vec<ProtoField> {
        vec![ #(#variants_tokens,)* ]
      }
    }

    impl #enum_name {
      #[track_caller]
      pub fn to_oneof() -> Oneof {
        let mut options: Vec<ProtoOption> = #options;

        #required_option_tokens

        Oneof {
          name: #proto_name.into(),
          fields: Self::fields(),
          options,
        }
      }
    }
  };

  Ok(output_tokens)
}

// pub(crate) fn process_oneof_derive_shadow(item: &mut ItemEnum) -> Result<TokenStream2, Error> {
//   let ItemEnum {
//     attrs,
//     ident: enum_name,
//     variants,
//     ..
//   } = item;
//
//   let OneofAttrs {
//     options,
//     name: proto_name,
//     required,
//   } = process_oneof_attrs(enum_name, attrs, false);
//
//   let prost_derive: Attribute = parse_quote!(#[derive(prost::Oneof)]);
//
//   attrs.push(prost_derive);
//
//   let mut variants_tokens: Vec<TokenStream2> = Vec::new();
//
//   for variant in variants {
//     let field_attrs = process_derive_field_attrs(&variant.ident, &variant.attrs)?;
//
//     let FieldAttrs {
//       tag,
//       validator,
//       options,
//       name,
//       kind,
//       ..
//     } = field_attrs;
//
//     let variant_type = if let Fields::Unnamed(variant_fields) = &variant.fields {
//       if variant_fields.unnamed.len() != 1 {
//         panic!("Oneof variants must contain a single value");
//       }
//
//       let type_ = &variant_fields.unnamed.first().unwrap().ty;
//
//       TypeInfo::from_type(type_, kind)?
//     } else {
//       panic!("Enum can only have one unnamed field")
//     };
//
//     // TODO
//     let proto_type = ProtoType::String;
//
//     let prost_attr_tokens =
//       ProstAttrs::from_type_info(&variant_type.rust_type, proto_type.clone(), tag);
//
//     let prost_attr: Attribute = parse_quote!(#prost_attr_tokens);
//
//     variant.attrs.push(prost_attr);
//
//     let validator_tokens = if let Some(validator) = validator {
//       variant_type.validator_tokens(&validator, &proto_type)
//     } else {
//       quote! { None }
//     };
//
//     let full_type_path = quote! {};
//
//     variants_tokens.push(quote! {
//       ProtoField {
//         name: #name.to_string(),
//         options: #options,
//         type_: <#full_type_path as AsProtoType>::proto_type(),
//         validator: #validator_tokens,
//         tag: #tag,
//       }
//     });
//   }
//
//   let required_option_tokens = required.then(|| quote! { options.push(oneof_required()); });
//
//   let output_tokens = quote! {
//     impl ProtoOneof for #enum_name {
//       fn fields() -> Vec<ProtoField> {
//         vec![ #(#variants_tokens,)* ]
//       }
//     }
//
//     impl #enum_name {
//       #[track_caller]
//       pub fn to_oneof() -> Oneof {
//         let mut options: Vec<ProtoOption> = #options;
//
//         #required_option_tokens
//
//         Oneof {
//           name: #proto_name.into(),
//           fields: Self::fields(),
//           options,
//         }
//       }
//     }
//   };
//
//   Ok(output_tokens)
// }
