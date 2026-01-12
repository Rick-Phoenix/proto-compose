use crate::*;

pub struct OneofCtx<'a, T: Borrow<FieldData>> {
  pub oneof_attrs: &'a OneofAttrs,
  pub orig_enum_ident: &'a Ident,
  pub shadow_enum_ident: Option<&'a Ident>,
  pub non_ignored_variants: Vec<T>,
  pub tags: Vec<ParsedNum>,
}

impl<'a, T: Borrow<FieldData>> OneofCtx<'a, T> {
  pub fn proto_enum_ident(&self) -> &'a Ident {
    self
      .shadow_enum_ident
      .unwrap_or(self.orig_enum_ident)
  }
}

pub fn process_oneof_proc_macro(mut item: ItemEnum, macro_attrs: TokenStream2) -> TokenStream2 {
  let oneof_attrs = match process_oneof_attrs(&item.ident, macro_attrs, &item.attrs) {
    Ok(attrs) => attrs,
    Err(e) => {
      let err = e.into_compile_error();

      return quote! {
        #item
        #err
      };
    }
  };

  // prost::Oneof already implements Debug and Default
  let mut proto_derives = if cfg!(feature = "cel") {
    quote! {
      #[derive(::prost::Oneof, Clone, PartialEq, ::prelude::CelOneof)]
      #[cel(cel_crate = ::prelude::cel, proto_types_crate = ::prelude::proto_types)]
    }
  } else {
    quote! { #[derive(::prost::Oneof, Clone, PartialEq)] }
  };

  if oneof_attrs.is_proxied {
    let mut shadow_enum = create_shadow_enum(&item);

    let impls = match oneof_shadow_proc_macro(&mut item, &mut shadow_enum, &oneof_attrs) {
      Ok(impls) => impls,
      Err(e) => {
        proto_derives = TokenStream2::new();

        FallbackImpls {
          error: e,
          orig_ident: &item.ident,
          shadow_ident: Some(&shadow_enum.ident),
          kind: InputItemKind::Oneof,
        }
        .generate_fallback_impls()
      }
    };

    let shadow_enum_derives = oneof_attrs
      .shadow_derives
      .map(|list| quote! { #[#list] });

    quote! {
      #[allow(clippy::derive_partial_eq_without_eq)]
      #[derive(::prelude::macros::Oneof)]
      #item

      #[allow(clippy::derive_partial_eq_without_eq)]
      #proto_derives
      #shadow_enum_derives
      #shadow_enum

      #impls
    }
  } else {
    let impls = match oneof_direct_proc_macro(&mut item, &oneof_attrs) {
      Ok(impls) => impls,
      Err(e) => {
        proto_derives = TokenStream2::new();

        FallbackImpls {
          error: e,
          orig_ident: &item.ident,
          shadow_ident: None,
          kind: InputItemKind::Oneof,
        }
        .generate_fallback_impls()
      }
    };

    quote! {
      #[allow(clippy::derive_partial_eq_without_eq)]
      #[derive(::prelude::macros::Oneof)]
      #proto_derives
      #item

      #impls
    }
  }
}

pub(crate) fn oneof_shadow_proc_macro(
  item: &mut ItemEnum,
  shadow_enum: &mut ItemEnum,
  oneof_attrs: &OneofAttrs,
) -> Result<TokenStream2, Error> {
  let orig_enum_ident = &item.ident;
  let shadow_enum_ident = &shadow_enum.ident;

  let mut ignored_variants: Vec<Ident> = Vec::new();

  let mut proto_conversion_data = ProtoConversionImpl {
    source_ident: orig_enum_ident,
    target_ident: shadow_enum_ident,
    kind: InputItemKind::Oneof,
    into_proto: ConversionData::new(oneof_attrs.into_proto.as_ref()),
    from_proto: ConversionData::new(oneof_attrs.from_proto.as_ref()),
  };

  let mut manually_set_tags: Vec<ParsedNum> = Vec::new();
  let mut fields_data: Vec<FieldDataKind> = Vec::new();

  for src_variant in item.variants.iter_mut() {
    let field_data_kind = process_field_data(FieldOrVariant::Variant(src_variant))?;
    proto_conversion_data.handle_field_conversions(&field_data_kind);

    match &field_data_kind {
      FieldDataKind::Ignored { ident, .. } => ignored_variants.push(ident.clone()),
      FieldDataKind::Normal(data) => {
        if let Some(tag) = data.tag {
          manually_set_tags.push(tag);
        }
      }
    };

    fields_data.push(field_data_kind);
  }

  sort_and_check_invalid_tags(&mut manually_set_tags, &ReservedNumbers::default())?;

  for (dst_variant, field_attrs) in shadow_enum
    .variants
    .iter_mut()
    .zip(fields_data.iter())
  {
    // Skipping ignored variants
    let FieldDataKind::Normal(field_attrs) = field_attrs else {
      continue;
    };

    if field_attrs.tag.is_none() {
      bail!(dst_variant.ident, "Tags in oneofs must be set manually");
    };

    let prost_attr = field_attrs.as_prost_attr();
    dst_variant.attrs.push(prost_attr);

    let prost_compatible_type = field_attrs.output_proto_type(true);
    *dst_variant.type_mut()? = prost_compatible_type;
  }

  // We strip away the ignored variants from the shadow enum
  let shadow_variants = std::mem::take(&mut shadow_enum.variants);
  shadow_enum.variants = shadow_variants
    .into_iter()
    .filter(|var| !ignored_variants.contains(&var.ident))
    .collect();

  let non_ignored_variants: Vec<&FieldData> = fields_data
    .iter()
    .filter_map(|f| f.as_normal())
    .collect();

  let proto_conversion_impls = proto_conversion_data.generate_conversion_impls();

  let oneof_ctx = OneofCtx {
    oneof_attrs,
    orig_enum_ident,
    shadow_enum_ident: Some(shadow_enum_ident),
    non_ignored_variants,
    tags: manually_set_tags,
  };

  let oneof_schema_impl = oneof_ctx.generate_schema_impl();
  let consistency_checks_impl = oneof_ctx.generate_consistency_checks();
  let validator_impl = oneof_ctx.generate_validator();

  let wrapped_items =
    wrap_with_imports(&[oneof_schema_impl, proto_conversion_impls, validator_impl]);

  Ok(quote! {
    #wrapped_items
    #consistency_checks_impl
  })
}

pub(crate) fn oneof_direct_proc_macro(
  item: &mut ItemEnum,
  oneof_attrs: &OneofAttrs,
) -> Result<TokenStream2, Error> {
  let ItemEnum { variants, .. } = item;

  let mut manually_set_tags: Vec<ParsedNum> = Vec::new();
  let mut fields_data: Vec<FieldData> = Vec::new();

  for variant in variants.iter_mut() {
    let field_attrs = process_field_data(FieldOrVariant::Variant(variant))?;
    let variant_type = variant.type_()?;

    if let FieldDataKind::Normal(data) = field_attrs {
      if let Some(tag) = data.tag {
        manually_set_tags.push(tag);
      }

      if data.proto_field.is_enum() && !data.type_info.inner().is_int() {
        bail!(&data.type_info, "Enums must use `i32` in direct impls")
      }

      match data.type_info.type_.as_ref() {
        RustType::Box(_) => {
          if !data.proto_field.is_boxed_message() {
            bail!(
              variant_type,
              "Box can only be used for messages in a native prost oneof"
            );
          }
        }

        // For unknown types such as messages
        RustType::Other(_) => {}

        _ => {
          if !data.type_info.type_.is_primitive() && !data.type_info.type_.is_bytes() {
            bail!(
              variant_type,
              "Unsupported Oneof variant type. If you want to use a custom type, you must use a proxied oneof with custom conversions"
            )
          }
        }
      };

      fields_data.push(data);
    } else {
      bail!(
        variant.ident,
        "Cannot use `ignore` in direct impls. Use a proxied impl instead"
      );
    }
  }

  sort_and_check_invalid_tags(&mut manually_set_tags, &ReservedNumbers::default())?;

  for (variant, field_attrs) in variants.iter_mut().zip(fields_data.iter()) {
    if field_attrs.tag.is_none() {
      bail!(variant.ident, "Tags in oneofs must be set manually");
    };

    let prost_attr = field_attrs.as_prost_attr();
    variant.attrs.push(prost_attr);
  }

  let oneof_ctx = OneofCtx {
    oneof_attrs,
    orig_enum_ident: &item.ident,
    shadow_enum_ident: None,
    non_ignored_variants: fields_data,
    tags: manually_set_tags,
  };

  let oneof_schema_impl = oneof_ctx.generate_schema_impl();
  let consistency_checks_impl = oneof_ctx.generate_consistency_checks();
  let validator_impl = oneof_ctx.generate_validator();

  let wrapped_items = wrap_with_imports(&[oneof_schema_impl, validator_impl]);

  let output = quote! {
    #wrapped_items
    #consistency_checks_impl
  };

  Ok(output)
}
