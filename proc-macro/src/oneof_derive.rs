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
    process_oneof_derive_shadow(item, oneof_attrs)
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

    if field_attrs.is_ignored {
      return Err(spanned_error!(
        &variant.ident,
        "Oneof variants cannot be ignored in a direct impl"
      ));
    }

    let variant_type = if let Fields::Unnamed(variant_fields) = &variant.fields {
      if variant_fields.unnamed.len() != 1 {
        panic!("Oneof variants must contain a single value");
      }

      variant_fields.unnamed.first().unwrap().ty.clone()
    } else {
      panic!("Enum can only have one unnamed field")
    };

    let type_info = TypeInfo::from_type(&variant_type, field_attrs.kind.clone())?;

    if !matches!(type_info.rust_type, RustType::Normal(_)) {
      return Err(spanned_error!(variant_type, "Unsupported enum variant. If you want to use a custom type, you must use the proxied variant"));
    };

    let variant_proto_tokens = process_field(
      &mut FieldOrVariant::Variant(variant),
      field_attrs,
      &type_info,
      OutputType::Keep,
    )?;

    variants_tokens.push(variant_proto_tokens);
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

pub(crate) fn process_oneof_derive_shadow(
  item: &mut ItemEnum,
  oneof_attrs: OneofAttrs,
) -> Result<TokenStream2, Error> {
  let mut shadow_enum = create_shadow_enum(item);

  let mut output_tokens = TokenStream2::new();

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

  shadow_enum.attrs.push(prost_derive);

  let mut variants_tokens: Vec<TokenStream2> = Vec::new();
  let mut ignored_variants: Vec<Ident> = Vec::new();

  let orig_enum_variants = variants.iter_mut();
  let shadow_enum_variants = shadow_enum.variants.iter_mut();

  let orig_enum_ident = &enum_name;
  let shadow_enum_ident = &shadow_enum.ident;

  let mut from_proto = TokenStream2::new();
  let mut into_proto = TokenStream2::new();

  for (src_variant, dst_variant) in orig_enum_variants.zip(shadow_enum_variants) {
    let field_attrs = process_derive_field_attrs(&src_variant.ident, &src_variant.attrs)?;

    let variant_type = if let Fields::Unnamed(variant_fields) = &src_variant.fields {
      if variant_fields.unnamed.len() != 1 {
        panic!("Oneof variants must contain a single value");
      }

      variant_fields.unnamed.first().unwrap().ty.clone()
    } else {
      panic!("Enum can only have one unnamed field")
    };

    let type_info = TypeInfo::from_type(&variant_type, field_attrs.kind.clone())?;

    let variant_ident = &src_variant.ident;

    if field_attrs.is_ignored {
      ignored_variants.push(src_variant.ident.clone());
    } else {
      let variant_proto_tokens = process_field(
        &mut FieldOrVariant::Variant(dst_variant),
        field_attrs,
        &type_info,
        OutputType::Change,
      )?;

      variants_tokens.push(variant_proto_tokens);

      if oneof_attrs.into_proto.is_none() {
        let call = type_info.into_proto();

        let into_proto_call = quote! {
          #orig_enum_ident::#variant_ident(v) => #shadow_enum_ident::#variant_ident(v.#call),
        };

        into_proto.extend(into_proto_call);
      }
    }

    if oneof_attrs.from_proto.is_none() {
      let from_proto_call = type_info.from_proto();

      let from_proto_expr = quote! {
        #shadow_enum_ident::#variant_ident(v) => #orig_enum_ident::#variant_ident(v.#from_proto_call),
      };

      from_proto.extend(from_proto_expr);
    }
  }

  shadow_enum.variants = shadow_enum
    .variants
    .into_iter()
    .filter(|var| !ignored_variants.contains(&var.ident))
    .collect();

  let required_option_tokens = required.then(|| quote! { options.push(oneof_required()); });

  output_tokens.extend(quote! {
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
  });

  let from_proto_body = if let Some(expr) = &oneof_attrs.from_proto {
    match expr {
      PathOrClosure::Path(path) => quote! { #path(value) },
      PathOrClosure::Closure(closure) => quote! {
        prelude::apply(value, #closure)
      },
    }
  } else {
    quote! {
      match value {
        #from_proto
      }
    }
  };

  let from_proto_impl = quote! {
    impl From<#shadow_enum_ident> for #orig_enum_ident {
      fn from(value: #shadow_enum_ident) -> Self {
        #from_proto_body
      }
    }

    impl #orig_enum_ident {
      pub fn from_proto(proto: #shadow_enum_ident) -> Self {
        proto.into()
      }

      pub fn into_proto(self) -> #shadow_enum_ident {
        self.into()
      }
    }
  };

  let into_proto_body = if let Some(expr) = &oneof_attrs.into_proto {
    match expr {
      PathOrClosure::Path(path) => quote! { #path(value) },
      PathOrClosure::Closure(closure) => quote! {
        prelude::apply(value, #closure)
      },
    }
  } else {
    quote! {
      match value {
        #into_proto
      }
    }
  };

  let into_proto_impl = quote! {
    impl From<#orig_enum_ident> for #shadow_enum_ident {
      fn from(value: #orig_enum_ident) -> Self {
        #into_proto_body
      }
    }
  };

  output_tokens.extend(quote! {
    #shadow_enum

    #from_proto_impl
    #into_proto_impl

    impl ProtoOneof for #shadow_enum_ident {
      fn fields() -> Vec<ProtoField> {
        #orig_enum_ident::fields()
      }
    }

    impl #shadow_enum_ident {
      #[track_caller]
      pub fn to_oneof() -> Oneof {
        #orig_enum_ident::to_oneof()
      }
    }
  });

  Ok(output_tokens)
}
