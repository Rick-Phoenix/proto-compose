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
  let mut ignored_variants: Vec<&Ident> = Vec::new();

  let mut proto_conversion_impls = ProtoConversionImpl {
    source_ident: orig_enum_ident,
    target_ident: shadow_enum_ident,
    kind: ItemConversionKind::Enum,
    into_proto: ConversionData::new(&oneof_attrs.into_proto),
    from_proto: ConversionData::new(&oneof_attrs.from_proto),
  };

  let mut validator_tokens = TokenStream2::new();
  let mut cel_rules_collection: Vec<TokenStream2> = Vec::new();
  let mut cel_checks_tokens = TokenStream2::new();

  for (src_variant, dst_variant) in orig_enum_variants.zip(shadow_enum_variants) {
    let variant_ident = &src_variant.ident;

    let variant_type = if let Fields::Unnamed(variant_fields) = &src_variant.fields
      && variant_fields.unnamed.len() == 1
    {
      &variant_fields.unnamed.first().unwrap().ty
    } else {
      bail!(
        &src_variant.fields,
        "Oneof variants can only contain a single unnamed value"
      );
    };

    let rust_type = TypeInfo::from_type(variant_type)?;

    let field_attrs =
      match process_derive_field_attrs(variant_ident, &rust_type, &src_variant.attrs)? {
        FieldAttrData::Ignored { from_proto } => {
          ignored_variants.push(variant_ident);

          if !proto_conversion_impls
            .from_proto
            .has_custom_impl()
          {
            proto_conversion_impls.add_field_from_proto_impl(
              &from_proto,
              None,
              FieldConversionKind::EnumVariant {
                variant_ident,
                source_enum_ident: orig_enum_ident,
                target_enum_ident: shadow_enum_ident,
              },
            );
          }

          // We close the loop early if the field is ignored
          continue;
        }

        FieldAttrData::Normal(field_attrs) => *field_attrs,
      };

    let type_ctx = TypeContext::new(rust_type, &field_attrs.proto_field)?;

    let variant_proto_tokens = process_field(FieldCtx {
      field: &mut FieldOrVariant::Variant(dst_variant),
      field_attrs: &field_attrs,
      type_ctx: &type_ctx,
      field_ident: &src_variant.ident,
      validators_tokens: &mut validator_tokens,
      cel_rules: &mut cel_rules_collection,
      cel_checks: &mut cel_checks_tokens,
    })?;

    variants_tokens.push(variant_proto_tokens);

    if !proto_conversion_impls
      .into_proto
      .has_custom_impl()
    {
      proto_conversion_impls.add_field_into_proto_impl(
        &oneof_attrs.into_proto,
        &type_ctx,
        FieldConversionKind::EnumVariant {
          variant_ident,
          source_enum_ident: orig_enum_ident,
          target_enum_ident: shadow_enum_ident,
        },
      );
    }

    if !proto_conversion_impls
      .from_proto
      .has_custom_impl()
    {
      proto_conversion_impls.add_field_from_proto_impl(
        &field_attrs.from_proto,
        Some(&type_ctx),
        FieldConversionKind::EnumVariant {
          variant_ident,
          source_enum_ident: orig_enum_ident,
          target_enum_ident: shadow_enum_ident,
        },
      );
    }
  }

  // We strip away the ignored variants from the shadow enum
  shadow_enum.variants = shadow_enum
    .variants
    .into_iter()
    .filter(|var| !ignored_variants.contains(&&var.ident))
    .collect();

  let oneof_schema_impl = oneof_schema_impl(&oneof_attrs, orig_enum_ident, variants_tokens);

  let into_proto_impl = proto_conversion_impls.create_into_proto_impl();
  let from_proto_impl = proto_conversion_impls.create_from_proto_impl();
  let conversion_helpers = proto_conversion_impls.create_conversion_helpers();

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
    #conversion_helpers

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

  let mut validator_tokens = TokenStream2::new();
  let mut cel_rules_collection: Vec<TokenStream2> = Vec::new();
  let mut cel_checks_tokens = TokenStream2::new();

  for variant in variants {
    let variant_type = if let Fields::Unnamed(variant_fields) = &variant.fields
      && variant_fields.unnamed.len() == 1
    {
      &variant_fields.unnamed.first().unwrap().ty
    } else {
      bail!(
        &variant.fields,
        "Oneof variants must contain a single unnamed value"
      );
    };

    let rust_type = TypeInfo::from_type(variant_type)?;

    let field_attrs = match process_derive_field_attrs(&variant.ident, &rust_type, &variant.attrs)?
    {
      FieldAttrData::Ignored { .. } => {
        bail!(
          &variant.ident,
          "Oneof variants cannot be ignored in a direct impl"
        );
      }
      FieldAttrData::Normal(field_attrs) => *field_attrs,
    };

    let type_ctx = TypeContext::new(rust_type, &field_attrs.proto_field)?;

    match type_ctx.rust_type.type_.as_ref() {
      RustType::Box(_) => {
          if !matches!(type_ctx.proto_field, ProtoField::Single(ProtoType::Message { is_boxed: true, .. })) {
            bail!(variant_type, "Box can only be used for messages in a native oneof");
          }
        },
      RustType::Other(_) => {},

      _ => bail!(variant_type, "Unsupported Oneof variant type. If you want to use a custom type, you must use a proxied oneof with custom conversions"),
    };

    let variant_proto_tokens = process_field(FieldCtx {
      field_ident: &variant.ident.clone(),
      field: &mut FieldOrVariant::Variant(variant),
      field_attrs: &field_attrs,
      type_ctx: &type_ctx,
      validators_tokens: &mut validator_tokens,
      cel_rules: &mut cel_rules_collection,
      cel_checks: &mut cel_checks_tokens,
    })?;

    variants_tokens.push(variant_proto_tokens);
  }

  let oneof_schema_impl = oneof_schema_impl(&oneof_attrs, &item.ident, variants_tokens);

  Ok(oneof_schema_impl)
}
