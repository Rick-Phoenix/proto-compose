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
    no_prefix,
  } = process_derive_enum_attrs(enum_name, attrs).unwrap();

  let reserved_numbers_tokens = reserved_numbers.to_token_stream();

  let mut variants_tokens: Vec<TokenStream2> = Vec::new();
  let mut from_str_tokens = TokenStream2::new();
  let mut as_str_tokens = TokenStream2::new();

  let mut used_tags: Vec<i32> = Vec::new();
  for variant in variants.iter() {
    if let Some((_, expr)) = &variant.discriminant {
      let num = extract_i32(expr)?;

      used_tags.push(num);
    }
  }

  let unavailable_ranges = reserved_numbers.build_unavailable_ranges(&used_tags);
  let mut tag_allocator = TagAllocator::new(&unavailable_ranges);

  for (i, variant) in variants.iter_mut().enumerate() {
    if !variant.fields.is_empty() {
      bail!(variant, "Protobuf enums can only have unit variants");
    }

    let variant_ident = &variant.ident;

    let EnumVariantAttrs { options, name } =
      process_derive_enum_variants_attrs(&proto_name, variant_ident, &variant.attrs, no_prefix)?;

    from_str_tokens.extend(quote! {
      #name => Some(Self::#variant_ident),
    });

    as_str_tokens.extend(quote! {
      Self::#variant_ident => #name,
    });

    let tag = if let Some((_, expr)) = &variant.discriminant {
      let tag = extract_i32(expr)?;

      if i == 0 && tag != 0 {
        bail!(
          expr,
          "The first variant of a protobuf enum must have have a tag of 0"
        );
      }

      tag
    } else {
      let next_tag = if i == 0 { 0 } else { tag_allocator.next_tag() };

      let tag_expr: Expr = parse_quote!(#next_tag);
      variant.discriminant = Some((token::Eq::default(), tag_expr));

      next_tag
    };

    variants_tokens.push(quote! {
      EnumVariant { name: #name.to_string(), options: #options, tag: #tag, }
    });
  }

  let output_tokens = quote! {
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
      pub fn from_int_or_default(int: i32) -> Self {
        int.try_into().unwrap_or_default()
      }

      pub fn as_proto_name(&self) -> &'static str {
        match self {
          #as_str_tokens
        }
      }

      pub fn from_proto_name(name: &str) -> Option<Self> {
        match name {
          #from_str_tokens
          _ => None
        }
      }

      #[track_caller]
      pub fn to_enum() -> Enum {
        Enum {
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
