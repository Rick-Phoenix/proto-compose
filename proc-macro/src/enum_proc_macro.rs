use crate::*;

struct EnumVariantCtx {
  name: String,
  options: TokensOr<TokenStream2>,
  tag: i32,
  ident: Ident,
  deprecated: bool,
  span: Span,
}

#[derive(Default)]
struct EnumData {
  variants_data: Vec<EnumVariantCtx>,
  enum_attrs: EnumAttrs,
}

fn extract_enum_data(item: &mut ItemEnum) -> syn::Result<EnumData> {
  let ItemEnum {
    ident: enum_ident,
    variants,
    attrs,
    ..
  } = item;

  let enum_attrs = process_derive_enum_attrs(enum_ident, attrs)?;

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

  let unavailable_ranges =
    build_unavailable_ranges(&enum_attrs.reserved_numbers, &mut manually_set_tags)?;

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
    } = process_derive_enum_variants_attrs(&enum_attrs.name, variant_ident, &variant.attrs)?;

    if enum_attrs.reserved_names.contains(&name) {
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

  Ok(EnumData {
    variants_data,
    enum_attrs,
  })
}

pub fn enum_proc_macro(mut item: ItemEnum) -> TokenStream2 {
  let mut error: Option<TokenStream2> = None;

  let EnumData {
    variants_data,
    enum_attrs:
      EnumAttrs {
        reserved_names,
        reserved_numbers,
        options: enum_options,
        parent_message,
        name: proto_name,
        deprecated,
        ..
      },
  } = extract_enum_data(&mut item).unwrap_or_else(|e| {
    error = Some(e.into_compile_error());
    EnumData::default()
  });

  let enum_ident = &item.ident;

  let proto_name_method = if let Some(parent) = &parent_message {
    quote_spanned! {parent.span()=>
      static __FULL_NAME: ::prelude::Lazy<String> = ::prelude::Lazy::new(|| {
        ::prelude::format!("{}.{}", <#parent as ::prelude::ProtoMessage>::proto_name(), #proto_name)
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

  let rust_ident_str = enum_ident.to_string();

  let variants_tokens = if error.is_some() {
    quote! { unimplemented!() }
  } else {
    let tokens = variants_data.iter().map(|var| {
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
      ::prelude::EnumVariant { name: #name.into(), options: #options_tokens.into_iter().collect(), tag: #tag, }
    }
  });

    quote! { #(#tokens),* }
  };

  let from_str_impl = if error.is_some() {
    quote! { unimplemented!() }
  } else {
    let tokens = variants_data.iter().map(|var| {
      let EnumVariantCtx {
        name, ident, span, ..
      } = var;

      quote_spanned! {*span=>
        #name => Some(Self::#ident)
      }
    });

    quote! {
      match name {
        #(#tokens,)*
        _ => None
      }
    }
  };

  let as_str_impl = if error.is_some() {
    quote! { unimplemented!() }
  } else {
    let tokens = variants_data.iter().map(|var| {
      let EnumVariantCtx {
        name, ident, span, ..
      } = var;

      quote_spanned! {*span=>
        Self::#ident => #name
      }
    });

    quote! {
      match self {
        #(#tokens),*
      }
    }
  };

  let try_from_impl = if error.is_some() {
    quote! { unimplemented!() }
  } else {
    let tokens = variants_data.iter().map(|var| {
      let EnumVariantCtx {
        tag, ident, span, ..
      } = var;

      quote_spanned! {*span=>
        #tag => Ok(#enum_ident::#ident)
      }
    });

    quote! {
      match value {
        #(#tokens,)*
        _ => Err(::prost::UnknownEnumValue(value))
      }
    }
  };

  let first_variant_ident = &variants_data.first().as_ref().unwrap().ident;

  let options_tokens = options_tokens(Span::call_site(), &enum_options, deprecated);

  quote! {
    #[repr(i32)]
    #[derive(::prelude::macros::Enum, Hash, PartialEq, Eq, Debug, Clone, Copy)]
    #item

    ::prelude::register_proto_data! {
      ::prelude::RegistryEnum {
        parent_message: #parent_message_registry,
        package: __PROTO_FILE.package,
        enum_: || <#enum_ident as ::prelude::ProtoEnumSchema>::proto_schema()
      }
    }

    impl TryFrom<i32> for #enum_ident {
      type Error = ::prost::UnknownEnumValue;

      #[inline]
      fn try_from(value: i32) -> Result<Self, Self::Error> {
        #try_from_impl
      }
    }

    impl Default for #enum_ident {
      #[inline]
      fn default() -> Self {
        #enum_ident::#first_variant_ident
      }
    }

    impl From<#enum_ident> for i32 {
      #[inline]
      fn from(value: #enum_ident) -> i32 {
        value as i32
      }
    }

    impl ::prelude::ProtoValidation for #enum_ident {
      #[doc(hidden)]
      type Target = i32;
      #[doc(hidden)]
      type Stored = i32;
      #[doc(hidden)]
      type Validator = ::prelude::EnumValidator<#enum_ident>;
      #[doc(hidden)]
      type Builder = ::prelude::EnumValidatorBuilder<#enum_ident>;

      type UniqueStore<'a>
        = ::prelude::CopyHybridStore<i32>
      where
        Self: 'a;
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
          name: <Self as ::prelude::ProtoEnum>::proto_name().into(),
          file: __PROTO_FILE.name.into(),
          package: __PROTO_FILE.package.into(),
        }
      }

      #[inline]
      fn as_proto_name(&self) -> &'static str {
        #as_str_impl
      }

      #[inline]
      fn from_proto_name(name: &str) -> Option<Self> {
        #from_str_impl
      }

      fn proto_schema() -> ::prelude::Enum {
        ::prelude::Enum {
          short_name: #proto_name.into(),
          name: <Self as ::prelude::ProtoEnum>::proto_name().into(),
          file: __PROTO_FILE.name.into(),
          package: __PROTO_FILE.package.into(),
          variants: ::prelude::vec! [ #variants_tokens ],
          reserved_names: ::prelude::vec![ #(#reserved_names.into()),* ],
          reserved_numbers: #reserved_numbers,
          options: #options_tokens.into_iter().collect(),
          rust_path:  ::prelude::format!("::{}::{}", __PROTO_FILE.extern_path, #rust_ident_str).into()
        }
      }
    }

    #error
  }
}
