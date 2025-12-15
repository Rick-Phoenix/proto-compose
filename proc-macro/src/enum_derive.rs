use crate::*;

pub fn process_enum_derive(item: &mut ItemEnum) -> Result<TokenStream2, Error> {
  let ItemEnum {
    attrs,
    ident: enum_name,
    ..
  } = item;

  let enum_attrs = process_derive_enum_attrs(enum_name, attrs)?;

  match enum_attrs.backend {
    Backend::Prost => process_enum_derive_prost(item, enum_attrs),
    Backend::Protobuf => unimplemented!(),
  }
}

pub fn process_enum_derive_prost(
  item: &mut ItemEnum,
  enum_attrs: EnumAttrs,
) -> Result<TokenStream2, Error> {
  let ItemEnum {
    attrs,
    ident: enum_name,
    variants,
    ..
  } = item;

  let repr_attr: Attribute = parse_quote!(#[repr(i32)]);
  let prost_attr: Attribute = parse_quote!(#[derive(::prost::Enumeration)]);
  attrs.push(repr_attr);
  attrs.push(prost_attr);

  let EnumAttrs {
    reserved_names,
    reserved_numbers,
    options,
    name: proto_name,
    file,
    package,
    full_name,
    no_prefix,
    ..
  } = enum_attrs;

  let mut variants_tokens: Vec<TokenStream2> = Vec::new();
  let mut from_str_tokens = TokenStream2::new();
  let mut as_str_tokens = TokenStream2::new();

  let mut used_tags: Vec<i32> = Vec::new();
  for variant in variants.iter() {
    if let Some((_, expr)) = &variant.discriminant {
      let num = expr.as_int::<i32>()?;

      used_tags.push(num);
    }
  }

  let unavailable_ranges = reserved_numbers
    .clone()
    .build_unavailable_ranges(&used_tags);
  let mut tag_allocator = TagAllocator::new(&unavailable_ranges);

  for (i, variant) in variants.iter_mut().enumerate() {
    if !variant.fields.is_empty() {
      bail!(variant, "Protobuf enums can only have unit variants");
    }

    let variant_ident = &variant.ident;

    let EnumVariantAttrs { options, name } =
      process_derive_enum_variants_attrs(&proto_name, variant_ident, &variant.attrs, no_prefix)?;

    if reserved_names.contains(&name) {
      bail!(&variant, "Name `{name}` is reserved");
    }

    from_str_tokens.extend(quote! {
      #name => Some(Self::#variant_ident),
    });

    as_str_tokens.extend(quote! {
      Self::#variant_ident => #name,
    });

    let tag = if let Some((_, expr)) = &variant.discriminant {
      let tag = expr.as_int::<i32>()?;

      if i == 0 && tag != 0 {
        bail!(
          expr,
          "The first variant of a protobuf enum must have have a tag of 0"
        );
      }

      if reserved_numbers.contains(tag) {
        bail!(&variant, "Tag {tag} is reserved");
      }

      tag
    } else {
      let next_tag = if i == 0 {
        0
      } else {
        tag_allocator
          .next_tag()
          .map_err(|e| error!(&variant, "{e}"))?
      };

      let tag_expr: Expr = parse_quote!(#next_tag);
      variant.discriminant = Some((token::Eq::default(), tag_expr));

      next_tag
    };

    let options_tokens = tokens_or_default!(options, quote! { vec![] });

    variants_tokens.push(quote! {
      ::prelude::EnumVariant { name: #name, options: #options_tokens, tag: #tag, }
    });
  }

  let options_tokens = tokens_or_default!(options, quote! { vec![] });

  let output_tokens = quote! {
    impl #enum_name {
      pub fn from_int_or_default(int: i32) -> Self {
        int.try_into().unwrap_or_default()
      }
    }

    impl ::prelude::ProtoValidator<#enum_name> for #enum_name {
      type Target = i32;
      type Validator = ::prelude::EnumValidator<#enum_name>;
      type Builder = ::prelude::EnumValidatorBuilder<#enum_name>;

      fn builder() -> Self::Builder {
        ::prelude::EnumValidator::builder()
      }
    }

    impl ::prelude::AsProtoType for #enum_name {
      fn proto_type() -> ::prelude::ProtoType {
        ::prelude::ProtoType::Enum(
          <Self as ::prelude::ProtoEnum>::proto_path()
        )
      }
    }

    impl ::prelude::ProtoEnum for #enum_name {
      fn proto_path() -> ::prelude::ProtoPath {
        ::prelude::ProtoPath {
          name: #full_name,
          file: #file,
          package: #package,
        }
      }

      fn proto_schema() -> ::prelude::Enum {
        Self::proto_schema()
      }
    }

    impl #enum_name {
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

      pub fn proto_schema() -> ::prelude::Enum {
        ::prelude::Enum {
          name: #proto_name,
          full_name: #full_name,
          package: #package,
          file: #file,
          variants: vec! [ #(#variants_tokens,)* ],
          reserved_names: vec![ #(#reserved_names),* ],
          reserved_numbers: vec![ #reserved_numbers ],
          options: #options_tokens,
        }
      }
    }
  };

  Ok(output_tokens)
}
