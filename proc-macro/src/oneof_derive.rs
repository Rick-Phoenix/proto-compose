use crate::*;

pub fn process_oneof_derive(item: &mut ItemEnum) -> Result<TokenStream2, Error> {
  let oneof_attrs = process_oneof_attrs(&item.ident, &item.attrs)?;

  match oneof_attrs.backend {
    Backend::Prost => process_oneof_derive_prost(item, oneof_attrs),
    Backend::Protobuf => unimplemented!(),
  }
}

pub fn process_oneof_derive_prost(
  item: &mut ItemEnum,
  oneof_attrs: OneofAttrs,
) -> Result<TokenStream2, Error> {
  if oneof_attrs.direct {
    process_oneof_derive_direct(item, oneof_attrs)
  } else {
    process_oneof_derive_shadow(item, oneof_attrs)
  }
}

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

    let variant_type = if let Fields::Unnamed(variant_fields) = &src_variant.fields {
      if variant_fields.unnamed.len() != 1 {
        bail!(
          &src_variant.fields,
          "Oneof variants can only contain a single value"
        );
      }

      variant_fields.unnamed.first().unwrap().ty.clone()
    } else {
      bail!(
        &src_variant.fields,
        "Oneof variants can only contain a single unnamed value"
      );
    };

    let rust_type = TypeInfo::from_type(&variant_type)?;

    let field_data =
      process_derive_field_attrs(&src_variant.ident, &rust_type, &src_variant.attrs)?;

    let field_attrs = match field_data {
      FieldAttrData::Ignored { from_proto } => {
        ignored_variants.push(src_variant.ident.clone());

        let from_proto_expr = field_from_proto_expression(FromProto {
          custom_expression: &from_proto,
          kind: FieldConversionKind::EnumVariant {
            variant_ident,
            source_enum_ident: orig_enum_ident,
            target_enum_ident: shadow_enum_ident,
          },
          type_info: None,
        });

        from_proto_body.extend(from_proto_expr);

        continue;
      }
      FieldAttrData::Normal(field_attrs) => *field_attrs,
    };

    let type_ctx = TypeContext::from_type(rust_type, field_attrs.proto_field.clone())?;

    let variant_proto_tokens = process_field(
      &mut FieldOrVariant::Variant(dst_variant),
      field_attrs.clone(),
      &type_ctx,
    )?;

    variants_tokens.push(variant_proto_tokens);

    if oneof_attrs.into_proto.is_none() {
      let field_into_proto = field_into_proto_expression(IntoProto {
        custom_expression: &field_attrs.into_proto,
        kind: FieldConversionKind::EnumVariant {
          variant_ident: &src_variant.ident,
          source_enum_ident: orig_enum_ident,
          target_enum_ident: shadow_enum_ident,
        },
        type_info: &type_ctx,
      })?;

      into_proto_body.extend(field_into_proto);
    }

    if oneof_attrs.from_proto.is_none() {
      let from_proto_expr = field_from_proto_expression(FromProto {
        custom_expression: &field_attrs.from_proto,
        kind: FieldConversionKind::EnumVariant {
          variant_ident,
          source_enum_ident: orig_enum_ident,
          target_enum_ident: shadow_enum_ident,
        },
        type_info: Some(&type_ctx),
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

  let shadow_enum_derives = oneof_attrs
    .shadow_derives
    .map(|list| quote! { #[#list] });

  output_tokens.extend(quote! {
    #oneof_schema_impl

    #[derive(prost::Oneof, PartialEq, Clone, ::protocheck_proc_macro::TryIntoCelValue)]
    #shadow_enum_derives
    #shadow_enum

    #from_proto_impl
    #into_proto_impl

    impl ::prelude::ProtoOneof for #shadow_enum_ident {
      fn proto_schema() -> ::prelude::Oneof {
        <#orig_enum_ident as ::prelude::ProtoOneof>::proto_schema()
      }
    }
  });

  Ok(output_tokens)
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
    let variant_type = if let Fields::Unnamed(variant_fields) = &variant.fields {
      if variant_fields.unnamed.len() != 1 {
        bail!(
          &variant.fields,
          "Oneof variants must contain a single unnamed value"
        );
      }

      variant_fields.unnamed.first().unwrap().ty.clone()
    } else {
      bail!(
        &variant.fields,
        "Oneof variants must contain a single unnamed value"
      );
    };

    let rust_type = TypeInfo::from_type(&variant_type)?;

    let field_data = process_derive_field_attrs(&variant.ident, &rust_type, &variant.attrs)?;

    let field_attrs = match field_data {
      FieldAttrData::Ignored { .. } => {
        bail!(
          &variant.ident,
          "Oneof variants cannot be ignored in a direct impl"
        );
      }
      FieldAttrData::Normal(field_attrs) => *field_attrs,
    };

    let type_ctx = TypeContext::from_type(rust_type, field_attrs.proto_field.clone())?;

    match type_ctx.rust_type.type_.as_ref() {
      RustType::Box(_) => {
          if !matches!(type_ctx.proto_field, ProtoField::Single(ProtoType::Message { is_boxed: true, .. })) {
            bail!(variant_type, "Box can only be used for messages in a native oneof");
          }
        },
      RustType::Other(_) => {},
      _ => bail!(variant_type, "Unsupported Oneof variant type. If you want to use a custom type, you must use a proxied oneof with custom conversions"),
    };

    let variant_proto_tokens = process_field(
      &mut FieldOrVariant::Variant(variant),
      field_attrs,
      &type_ctx,
    )?;

    variants_tokens.push(variant_proto_tokens);
  }

  let oneof_schema_impl = oneof_schema_impl(&oneof_attrs, &item.ident, variants_tokens);

  Ok(oneof_schema_impl)
}
