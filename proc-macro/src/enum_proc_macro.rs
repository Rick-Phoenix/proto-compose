use crate::*;

struct EnumVariantCtx {
  name: String,
  options: TokensOr<TokenStream2>,
  tag: i32,
  ident: Ident,
  deprecated: bool,
}

pub fn enum_proc_macro(mut item: ItemEnum) -> TokenStream2 {
  let schema_impls = match enum_schema_impls(&mut item) {
    Ok(impls) => impls,
    Err(e) => {
      let err = e.into_compile_error();
      let fallback_impls = fallback_schema_impl(&item.ident);

      quote! {
        #fallback_impls
        #err
      }
    }
  };

  quote! {
    #[repr(i32)]
    #[derive(::prelude::macros::Enum, Hash, PartialEq, Eq, Debug, Clone, Copy)]
    #item

    #schema_impls
  }
}

fn enum_schema_impls(item: &mut ItemEnum) -> Result<TokenStream2, Error> {
  let ItemEnum {
    ident: enum_name,
    variants,
    attrs,
    ..
  } = item;

  let EnumAttrs {
    reserved_names,
    reserved_numbers,
    options: enum_options,
    name: proto_name,
    no_prefix,
    parent_message,
    extern_path,
    deprecated,
    ..
  } = process_derive_enum_attrs(enum_name, attrs)?;

  let mut variants_data: Vec<EnumVariantCtx> = Vec::new();
  let mut manually_set_tags: Vec<ManuallySetTag> = Vec::new();

  for variant in variants.iter() {
    if let Some((_, expr)) = &variant.discriminant {
      let num = expr.as_int::<i32>()?;

      manually_set_tags.push(ManuallySetTag {
        tag: num,
        field_span: variant.ident.span(),
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

    let EnumVariantAttrs {
      options,
      name,
      deprecated,
    } = process_derive_enum_variants_attrs(&proto_name, variant_ident, &variant.attrs, no_prefix)?;

    if reserved_names.contains(&name) {
      bail!(&variant, "Name `{name}` is reserved");
    }

    let tag = if let Some((_, expr)) = &variant.discriminant {
      let tag = expr.as_int::<i32>()?;

      if i == 0 && tag != 0 {
        bail!(
          expr,
          "The first variant of a protobuf enum must have have a tag of 0"
        );
      }

      tag
    } else {
      let next_tag = if i == 0 {
        0
      } else {
        tag_allocator.next_tag(variant.ident.span())?
      };

      let tag_expr: Expr = parse_quote!(#next_tag);
      variant.discriminant = Some((token::Eq::default(), tag_expr));

      next_tag
    };

    variants_data.push(EnumVariantCtx {
      name,
      options,
      tag,
      ident: variant_ident.clone(),
      deprecated,
    });
  }

  let proto_name_method = if let Some(parent) = &parent_message {
    quote! {
      static __FULL_NAME: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
        format!("{}.{}", <#parent as ::prelude::ProtoMessage>::proto_name(), #proto_name).into()
      });

      &*__FULL_NAME
    }
  } else {
    quote! { #proto_name }
  };

  let parent_message_registry = if let Some(parent) = &parent_message {
    quote! { Some(|| <#parent as ::prelude::ProtoMessage>::proto_name()) }
  } else {
    quote! { None }
  };

  let rust_path_field = if let Some(extern_path) = extern_path {
    quote! { #extern_path.to_string() }
  } else {
    let rust_ident_str = enum_name.to_string();

    quote! { format!("::{}::{}", __PROTO_FILE.extern_path, #rust_ident_str) }
  };

  let variants_tokens = variants_data.iter().map(|var| {
    let EnumVariantCtx {
      name,
      options,
      tag,
      deprecated,
      ..
    } = var;

    let options_tokens = options_tokens(options, *deprecated);

    quote! {
      ::prelude::EnumVariant { name: #name, options: #options_tokens.into_iter().collect(), tag: #tag, }
    }
  });

  let from_str_impl = variants_data.iter().map(|var| {
    let EnumVariantCtx { name, ident, .. } = var;

    quote! {
      #name => Some(Self::#ident)
    }
  });

  let as_str_impl = variants_data.iter().map(|var| {
    let EnumVariantCtx { name, ident, .. } = var;

    quote! {
      Self::#ident => #name
    }
  });

  let try_from_impl = variants_data.iter().map(|var| {
    let EnumVariantCtx { ident, tag, .. } = var;

    quote! {
      #tag => Ok(#enum_name::#ident)
    }
  });

  let first_variant_ident = &variants_data.first().as_ref().unwrap().ident;

  let options_tokens = options_tokens(&enum_options, deprecated);

  Ok(quote! {
    ::prelude::inventory::submit! {
      ::prelude::RegistryEnum {
        parent_message: #parent_message_registry,
        package: __PROTO_FILE.package,
        enum_: || <#enum_name as ::prelude::ProtoEnumSchema>::proto_schema()
      }
    }

    impl TryFrom<i32> for #enum_name {
      type Error = ::prost::UnknownEnumValue;

      fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
          #(#try_from_impl,)*
          _ => Err(::prost::UnknownEnumValue(value))
        }
      }
    }

    impl Default for #enum_name {
      fn default() -> Self {
        #enum_name::#first_variant_ident
      }
    }

    impl From<#enum_name> for i32 {
      fn from(value: #enum_name) -> i32 {
        value as i32
      }
    }

    impl ::prelude::ProtoValidator for #enum_name {
      #[doc(hidden)]
      type Target = i32;
      #[doc(hidden)]
      type Validator = ::prelude::EnumValidator<#enum_name>;
      #[doc(hidden)]
      type Builder = ::prelude::EnumValidatorBuilder<#enum_name>;
    }

    impl ::prelude::AsProtoType for #enum_name {
      fn proto_type() -> ::prelude::ProtoType {
        ::prelude::ProtoType::Enum(
          <Self as ::prelude::ProtoEnumSchema>::proto_path()
        )
      }
    }

    impl ::prelude::ProtoEnum for #enum_name {
      fn proto_name() -> &'static str {
        #proto_name_method
      }
    }

    impl ::prelude::ProtoEnumSchema for #enum_name {
      fn proto_path() -> ::prelude::ProtoPath {
        ::prelude::ProtoPath {
          name: <Self as ::prelude::ProtoEnum>::proto_name(),
          file: __PROTO_FILE.name,
          package: __PROTO_FILE.package,
        }
      }

      #[inline]
      fn as_proto_name(&self) -> &'static str {
        match self {
          #(#as_str_impl),*
        }
      }

      #[inline]
      fn from_proto_name(name: &str) -> Option<Self> {
        match name {
          #(#from_str_impl,)*
          _ => None
        }
      }

      fn proto_schema() -> ::prelude::Enum {
        ::prelude::Enum {
          short_name: #proto_name,
          name: <Self as ::prelude::ProtoEnum>::proto_name(),
          file: __PROTO_FILE.name,
          package: __PROTO_FILE.package,
          variants: vec! [ #(#variants_tokens,)* ],
          reserved_names: vec![ #(#reserved_names),* ],
          reserved_numbers: vec![ #reserved_numbers ],
          options: #options_tokens.into_iter().collect(),
          rust_path: #rust_path_field
        }
      }
    }
  })
}

fn fallback_schema_impl(enum_name: &Ident) -> TokenStream2 {
  quote! {
    impl ::prelude::ProtoValidator for #enum_name {
      type Target = i32;
      type Validator = ::prelude::EnumValidator<#enum_name>;
      type Builder = ::prelude::EnumValidatorBuilder<#enum_name>;
    }

    impl ::prelude::AsProtoType for #enum_name {
      fn proto_type() -> ::prelude::ProtoType {
        unimplemented!()
      }
    }

    impl ::prelude::ProtoEnumSchema for #enum_name {
      fn proto_name() -> &'static str {
        unimplemented!()
      }

      fn proto_path() -> ::prelude::ProtoPath {
        unimplemented!()
      }

      #[inline]
      fn as_proto_name(&self) -> &'static str {
        unimplemented!()
      }

      #[inline]
      fn from_proto_name(name: &str) -> Option<Self> {
        unimplemented!()
      }

      fn proto_schema() -> ::prelude::Enum {
        unimplemented!()
      }
    }
  }
}
