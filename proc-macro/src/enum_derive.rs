use crate::*;

pub(crate) fn process_enum_derive(item: &mut ItemEnum) -> Result<TokenStream2, Error> {
  let ItemEnum {
    attrs,
    ident: enum_name,
    variants,
    ..
  } = item;

  let repr_attr: Attribute = parse_quote!(#[repr(i32)]);
  attrs.push(repr_attr);

  let EnumAttrs {
    reserved_names,
    reserved_numbers,
    options,
    name: proto_name,
    file,
    package,
    full_name,
  } = process_derive_enum_attrs(&enum_name, &attrs).unwrap();

  let reserved_numbers_tokens = reserved_numbers.to_token_stream();

  let mut variants_tokens: Vec<TokenStream2> = Vec::new();

  let mut used_tags: Vec<i32> = Vec::new();

  for variant in variants.iter() {
    if let Some((_, expr)) = &variant.discriminant {
      let num = extract_i32(expr)?;

      used_tags.push(num);
    }
  }

  let unavailable_ranges = reserved_numbers.build_unavailable_ranges(&used_tags);
  let mut tag_allocator = TagAllocator::new(&unavailable_ranges);

  for variant in variants {
    if !variant.fields.is_empty() {
      panic!("Must be a unit variant");
    }

    let EnumVariantAttrs { options, name } =
      process_derive_enum_variants_attrs(&proto_name, &variant.ident, &variant.attrs)?;

    let tag = if let Some((_, expr)) = &variant.discriminant {
      extract_i32(expr)?
    } else {
      let next_tag = tag_allocator.next_tag();

      let tag_expr: Expr = parse_quote!(#next_tag);
      variant.discriminant = Some((token::Eq::default(), tag_expr));

      next_tag
    };

    variants_tokens.push(quote! {
      EnumVariant { name: #name.to_string(), options: #options, tag: #tag, }
    });
  }

  let output_tokens = quote! {
    impl ProtoEnumTrait for #enum_name {}

    impl ProtoValidator<#enum_name> for ValidatorMap {
      type Builder = EnumValidatorBuilder;

      fn builder() -> Self::Builder {
        EnumValidator::builder()
      }
    }

    impl AsProtoType for #enum_name {
      fn proto_type() -> ProtoType {
        ProtoType::Single(TypeInfo {
          name: #full_name,
          path: Some(ProtoPath {
            file: #file.into(),
            package: #package.into()
          })
        })
      }
    }

    impl #enum_name {
      #[track_caller]
      pub fn to_enum() -> ProtoEnum {
        ProtoEnum {
          name: #proto_name.into(),
          full_name: #full_name,
          package: #package.into(),
          file: #file.into(),
          variants: vec! [ #(#variants_tokens,)* ],
          reserved_names: #reserved_names,
          reserved_numbers: vec![ #reserved_numbers_tokens ],
          options: #options,
        }
      }
    }
  };

  Ok(output_tokens)
}
