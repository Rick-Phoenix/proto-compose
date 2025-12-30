use crate::*;

pub fn process_enum_derive(item: &mut ItemEnum) -> Result<TokenStream2, Error> {
  let ItemEnum {
    ident: enum_name,
    variants,
    attrs,
    ..
  } = item;

  let EnumAttrs {
    reserved_names,
    reserved_numbers,
    options,
    name: proto_name,
    no_prefix,
    parent_message,
    extern_path,
    ..
  } = process_derive_enum_attrs(enum_name, attrs)?;

  let mut variants_tokens: Vec<TokenStream2> = Vec::new();
  let mut from_str_tokens = TokenStream2::new();
  let mut as_str_tokens = TokenStream2::new();
  let mut manually_set_tags: Vec<ManuallySetTag> = Vec::new();

  for variant in variants.iter() {
    if let Some((_, expr)) = &variant.discriminant {
      let num = expr.as_int::<i32>()?;

      manually_set_tags.push(ManuallySetTag {
        tag: num,
        field_span: variant.span(),
      });
    }
  }

  let unavailable_ranges = build_unavailable_ranges(&reserved_numbers, &mut manually_set_tags)?;

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

  let full_name_method = if let Some(parent) = &parent_message {
    quote! {
      static __FULL_NAME: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
        format!("{}.{}", #parent::full_name(), #proto_name).into()
      });

      &*__FULL_NAME
    }
  } else {
    quote! { #proto_name }
  };

  let parent_message_registry = if let Some(parent) = &parent_message {
    quote! { Some(|| #parent::full_name()) }
  } else {
    quote! { None }
  };

  let rust_path_field = if let Some(extern_path) = extern_path {
    quote! { #extern_path.to_string() }
  } else {
    let rust_ident_str = enum_name.to_string();

    quote! { format!("::{}::{}", __PROTO_FILE.extern_path, #rust_ident_str) }
  };

  let output_tokens = quote! {
    ::prelude::inventory::submit! {
      ::prelude::RegistryEnum {
        parent_message: #parent_message_registry,
        package: __PROTO_FILE.package,
        enum_: || #enum_name::proto_schema()
      }
    }

    impl #enum_name {
      pub fn from_int_or_default(int: i32) -> Self {
        int.try_into().unwrap_or_default()
      }
    }

    impl ::prelude::ProtoValidator for #enum_name {
      type Target = i32;
      type Validator = ::prelude::EnumValidator<#enum_name>;
      type Builder = ::prelude::EnumValidatorBuilder<#enum_name>;

      fn validator_builder() -> Self::Builder {
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
      fn full_name() -> &'static str {
        #full_name_method
      }

      fn proto_path() -> ::prelude::ProtoPath {
        ::prelude::ProtoPath {
          name: Self::full_name(),
          file: __PROTO_FILE.file,
          package: __PROTO_FILE.package,
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
          full_name: Self::full_name(),
          file: __PROTO_FILE.file,
          package: __PROTO_FILE.package,
          variants: vec! [ #(#variants_tokens,)* ],
          reserved_names: vec![ #(#reserved_names),* ],
          reserved_numbers: vec![ #reserved_numbers ],
          options: #options_tokens,
          rust_path: #rust_path_field
        }
      }
    }
  };

  Ok(output_tokens)
}
