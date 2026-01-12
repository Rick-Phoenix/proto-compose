use crate::*;

struct EnumVariantCtx {
  name: String,
  options: TokensOr<TokenStream2>,
  tag: i32,
  ident: Ident,
  deprecated: bool,
  span: Span,
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
    ident: enum_ident,
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
  } = process_derive_enum_attrs(enum_ident, attrs)?;

  let mut variants_data: Vec<EnumVariantCtx> = Vec::new();
  let mut manually_set_tags: Vec<ParsedNum> = Vec::new();

  for variant in variants.iter() {
    if let Some((_, expr)) = &variant.discriminant {
      let num = expr.as_int::<i32>()?;

      manually_set_tags.push(ParsedNum {
        num,
        span: variant.ident.span(),
      });
    }
  }

  let unavailable_ranges = build_unavailable_ranges(&reserved_numbers, &mut manually_set_tags)?;

  let mut tag_allocator = TagAllocator::new(&unavailable_ranges);

  for (i, variant) in variants.iter_mut().enumerate() {
    let variant_ident = &variant.ident;

    if !variant.fields.is_empty() {
      bail!(variant_ident, "Protobuf enums can only have unit variants");
    }

    let EnumVariantAttrs {
      options,
      name,
      deprecated,
    } = process_derive_enum_variants_attrs(&proto_name, variant_ident, &variant.attrs, no_prefix)?;

    if reserved_names.contains(&name) {
      bail!(variant_ident, "Name `{name}` is reserved");
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
      span: variant_ident.span(),
    });
  }

  let proto_name_method = if let Some(parent) = &parent_message {
    quote_spanned! {parent.span()=>
      static __FULL_NAME: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
        format!("{}.{}", <#parent as ::prelude::ProtoMessage>::proto_name(), #proto_name)
      });

      &*__FULL_NAME
    }
  } else {
    quote! { #proto_name }
  };

  let parent_message_registry = if let Some(parent) = &parent_message {
    quote_spanned! {parent.span()=> Some(|| <#parent as ::prelude::ProtoMessage>::proto_name()) }
  } else {
    quote! { None }
  };

  let rust_path_field = if let Some(extern_path) = extern_path {
    quote_spanned! {extern_path.span()=> #extern_path.to_string() }
  } else {
    let rust_ident_str = enum_ident.to_string();

    quote! { format!("::{}::{}", __PROTO_FILE.extern_path, #rust_ident_str) }
  };

  let variants_tokens = variants_data.iter().map(|var| {
    let EnumVariantCtx {
      name,
      options,
      tag,
      deprecated,
      span,
      ..
    } = var;

    let options_tokens = options_tokens(*span, options, *deprecated);

    quote_spanned! {*span=>
      ::prelude::EnumVariant { name: #name, options: #options_tokens.into_iter().collect(), tag: #tag, }
    }
  });

  let from_str_impl = variants_data.iter().map(|var| {
    let EnumVariantCtx {
      name, ident, span, ..
    } = var;

    quote_spanned! {*span=>
      #name => Some(Self::#ident)
    }
  });

  let as_str_impl = variants_data.iter().map(|var| {
    let EnumVariantCtx {
      name, ident, span, ..
    } = var;

    quote_spanned! {*span=>
      Self::#ident => #name
    }
  });

  let try_from_impl = variants_data.iter().map(|var| {
    let EnumVariantCtx {
      ident, tag, span, ..
    } = var;

    quote_spanned! {*span=>
      #tag => Ok(#enum_ident::#ident)
    }
  });

  let first_variant_ident = &variants_data.first().as_ref().unwrap().ident;

  let options_tokens = options_tokens(Span::call_site(), &enum_options, deprecated);

  Ok(quote! {
    ::prelude::inventory::submit! {
      ::prelude::RegistryEnum {
        parent_message: #parent_message_registry,
        package: __PROTO_FILE.package,
        enum_: || <#enum_ident as ::prelude::ProtoEnumSchema>::proto_schema()
      }
    }

    impl TryFrom<i32> for #enum_ident {
      type Error = ::prost::UnknownEnumValue;

      fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
          #(#try_from_impl,)*
          _ => Err(::prost::UnknownEnumValue(value))
        }
      }
    }

    impl Default for #enum_ident {
      fn default() -> Self {
        #enum_ident::#first_variant_ident
      }
    }

    impl From<#enum_ident> for i32 {
      fn from(value: #enum_ident) -> i32 {
        value as i32
      }
    }

    impl ::prelude::ProtoValidator for #enum_ident {
      #[doc(hidden)]
      type Target = i32;
      #[doc(hidden)]
      type Validator = ::prelude::EnumValidator<#enum_ident>;
      #[doc(hidden)]
      type Builder = ::prelude::EnumValidatorBuilder<#enum_ident>;
    }

    impl ::prelude::AsProtoType for #enum_ident {
      fn proto_type() -> ::prelude::ProtoType {
        ::prelude::ProtoType::Enum(
          <Self as ::prelude::ProtoEnumSchema>::proto_path()
        )
      }
    }

    impl ::prelude::ProtoEnum for #enum_ident {
      fn proto_name() -> &'static str {
        #proto_name_method
      }
    }

    impl ::prelude::ProtoEnumSchema for #enum_ident {
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
          reserved_numbers: #reserved_numbers,
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
