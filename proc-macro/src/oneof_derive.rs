use crate::*;

pub(crate) fn process_oneof_derive_shadow(
  item: &mut ItemEnum,
  oneof_attrs: OneofAttrs,
) -> Result<TokenStream2, Error> {
  let mut shadow_enum = create_shadow_enum(item);

  let orig_enum_ident = &item.ident;
  let shadow_enum_ident = &shadow_enum.ident;

  let mut output_tokens = TokenStream2::new();
  let mut variants_tokens: Vec<TokenStream2> = Vec::new();

  let orig_enum_variants = item.variants.iter_mut();
  let shadow_enum_variants = shadow_enum.variants.iter_mut();
  let mut ignored_variants: Vec<Ident> = Vec::new();

  let mut from_proto_body = TokenStream2::new();
  let mut into_proto_body = TokenStream2::new();

  for (src_variant, dst_variant) in orig_enum_variants.zip(shadow_enum_variants) {
    let variant_ident = &src_variant.ident;

    let field_attrs = process_derive_field_attrs(&src_variant.ident, &src_variant.attrs)?;

    let variant_type = if let Fields::Unnamed(variant_fields) = &src_variant.fields {
      if variant_fields.unnamed.len() != 1 {
        panic!("Oneof variants must contain a single value");
      }

      variant_fields.unnamed.first().unwrap().ty.clone()
    } else {
      panic!("Enum can only have one unnamed field")
    };

    let type_info = TypeInfo::from_type(&variant_type, field_attrs.kind.clone(), orig_enum_ident)?;

    if field_attrs.is_ignored {
      ignored_variants.push(src_variant.ident.clone());
    } else {
      let variant_proto_tokens = process_field(
        &mut FieldOrVariant::Variant(dst_variant),
        field_attrs.clone(),
        &type_info,
        OutputType::Change,
      )?;

      variants_tokens.push(variant_proto_tokens);

      if oneof_attrs.into_proto.is_none() {
        let field_into_proto = field_into_proto_expression(FieldConversion {
          custom_expression: &field_attrs.into_proto,
          kind: FieldConversionKind::EnumVariant {
            variant_ident: &src_variant.ident,
            source_enum_ident: orig_enum_ident,
            target_enum_ident: shadow_enum_ident,
          },
          type_info: &type_info,
          is_ignored: field_attrs.is_ignored,
        })?;

        into_proto_body.extend(field_into_proto);
      }
    }

    if oneof_attrs.from_proto.is_none() {
      let from_proto_expr = field_from_proto_expression(FieldConversion {
        custom_expression: &field_attrs.from_proto,
        kind: FieldConversionKind::EnumVariant {
          variant_ident,
          source_enum_ident: orig_enum_ident,
          target_enum_ident: shadow_enum_ident,
        },
        type_info: &type_info,
        is_ignored: field_attrs.is_ignored,
      });

      from_proto_body.extend(from_proto_expr);
    }
  }

  shadow_enum.variants = shadow_enum
    .variants
    .into_iter()
    .filter(|var| !ignored_variants.contains(&var.ident))
    .collect();

  let oneof_schema_impl = oneof_schema_impl(&oneof_attrs, orig_enum_ident, variants_tokens);

  let into_proto_impl = into_proto_impl(ItemConversion {
    source_ident: orig_enum_ident,
    target_ident: shadow_enum_ident,
    kind: ItemConversionKind::Enum,
    custom_expression: &oneof_attrs.into_proto,
    conversion_tokens: into_proto_body,
  });

  let from_proto_impl = from_proto_impl(ItemConversion {
    source_ident: orig_enum_ident,
    target_ident: shadow_enum_ident,
    kind: ItemConversionKind::Enum,
    custom_expression: &oneof_attrs.from_proto,
    conversion_tokens: from_proto_body,
  });

  output_tokens.extend(quote! {
    #oneof_schema_impl

    #[derive(prost::Oneof, PartialEq, Clone)]
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

pub fn process_oneof_derive(item: &mut ItemEnum) -> Result<TokenStream2, Error> {
  let oneof_attrs = process_oneof_attrs(&item.ident, &item.attrs)?;

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
    attrs, variants, ..
  } = item;

  let prost_derive: Attribute = parse_quote!(#[derive(prost::Oneof, PartialEq, Clone)]);
  attrs.push(prost_derive);

  let mut variants_tokens: Vec<TokenStream2> = Vec::new();

  for variant in variants {
    let variant_ident = &variant.ident;

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

    let type_info = TypeInfo::from_type(&variant_type, field_attrs.kind.clone(), &item.ident)?;

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

  let oneof_schema_impl = oneof_schema_impl(&oneof_attrs, &item.ident, variants_tokens);

  Ok(oneof_schema_impl)
}
