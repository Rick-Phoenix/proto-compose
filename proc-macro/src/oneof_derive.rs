use crate::*;

#[derive(Default)]
pub struct OneofMacroAttrs {
  pub is_proxied: bool,
  pub no_auto_test: bool,
}

pub struct OneofCtx<'a, T: Borrow<FieldData>> {
  pub oneof_attrs: OneofAttrs,
  pub orig_enum_ident: &'a Ident,
  pub shadow_enum_ident: Option<&'a Ident>,
  pub non_ignored_variants: Vec<T>,
  pub tags: Vec<ManuallySetTag>,
}

impl<'a, T: Borrow<FieldData>> OneofCtx<'a, T> {
  pub fn proto_enum_ident(&self) -> &'a Ident {
    self
      .shadow_enum_ident
      .unwrap_or(self.orig_enum_ident)
  }
}

pub fn process_oneof_derive(
  item: &mut ItemEnum,
  macro_attrs: OneofMacroAttrs,
) -> Result<TokenStream2, Error> {
  let oneof_attrs = process_oneof_attrs(&item.ident, macro_attrs, &item.attrs)?;

  if oneof_attrs.is_proxied {
    process_oneof_derive_shadow(item, oneof_attrs)
  } else {
    process_oneof_derive_direct(item, oneof_attrs)
  }
}

pub(crate) fn process_oneof_derive_shadow(
  item: &mut ItemEnum,
  oneof_attrs: OneofAttrs,
) -> Result<TokenStream2, Error> {
  let mut shadow_enum = create_shadow_enum(item);

  let orig_enum_ident = &item.ident;
  let shadow_enum_ident = &shadow_enum.ident;

  let mut ignored_variants: Vec<Ident> = Vec::new();

  let mut proto_conversion_data = ProtoConversionImpl {
    source_ident: orig_enum_ident,
    target_ident: shadow_enum_ident,
    kind: InputItemKind::Oneof,
    into_proto: ConversionData::new(&oneof_attrs.into_proto),
    from_proto: ConversionData::new(&oneof_attrs.from_proto),
  };

  let mut manually_set_tags: Vec<ManuallySetTag> = Vec::new();
  let mut fields_data: Vec<FieldDataKind> = Vec::new();

  for src_variant in item.variants.iter_mut() {
    let field_data_kind = process_field_data(FieldOrVariant::Variant(src_variant))?;
    proto_conversion_data.handle_field_conversions(&field_data_kind);

    match &field_data_kind {
      FieldDataKind::Ignored { ident, .. } => ignored_variants.push(ident.clone()),
      FieldDataKind::Normal(data) => {
        if let Some(tag) = data.tag {
          manually_set_tags.push(ManuallySetTag {
            tag,
            field_span: src_variant.span(),
          });
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

    let Some(tag) = field_attrs.tag else {
      bail!(dst_variant, "Tags in oneofs must be set manually");
    };

    let prost_attr = field_attrs.proto_field.as_prost_attr(tag);
    dst_variant.attrs.push(prost_attr);

    let prost_compatible_type = field_attrs.proto_field.output_proto_type();
    *dst_variant.type_mut()? = prost_compatible_type;
  }

  // We strip away the ignored variants from the shadow enum
  shadow_enum.variants = shadow_enum
    .variants
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

  let wrapped_items = wrap_with_imports(vec![
    oneof_schema_impl,
    proto_conversion_impls,
    validator_impl,
  ]);

  // prost::Oneof already implements Debug
  let derives = if cfg!(feature = "cel") {
    quote! { #[derive(::prelude::prost::Oneof, PartialEq, Clone, ::protocheck_proc_macro::TryIntoCelValue)] }
  } else {
    quote! { #[derive(::prelude::prost::Oneof, PartialEq, Clone)] }
  };

  let shadow_enum_derives = oneof_ctx
    .oneof_attrs
    .shadow_derives
    .map(|list| quote! { #[#list] });

  Ok(quote! {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #derives
    #shadow_enum_derives
    #shadow_enum

    #wrapped_items
    #consistency_checks_impl
  })
}

pub(crate) fn process_oneof_derive_direct(
  item: &mut ItemEnum,
  oneof_attrs: OneofAttrs,
) -> Result<TokenStream2, Error> {
  let ItemEnum {
    attrs, variants, ..
  } = item;

  attrs.push(parse_quote!(#[allow(clippy::derive_partial_eq_without_eq)]));

  // prost::Oneof already implements Debug
  let prost_derive: Attribute = if cfg!(feature = "cel") {
    parse_quote!(#[derive(::prelude::prost::Oneof, PartialEq, Clone, ::protocheck_proc_macro::TryIntoCelValue)])
  } else {
    parse_quote!(#[derive(::prelude::prost::Oneof, PartialEq, Clone)])
  };

  attrs.push(prost_derive);

  let mut manually_set_tags: Vec<ManuallySetTag> = Vec::new();
  let mut fields_data: Vec<FieldData> = Vec::new();

  for variant in variants.iter_mut() {
    let field_attrs = process_field_data(FieldOrVariant::Variant(variant))?;
    let variant_type = variant.type_()?;

    if let FieldDataKind::Normal(data) = field_attrs {
      if let Some(tag) = data.tag {
        manually_set_tags.push(ManuallySetTag {
          tag,
          field_span: variant.span(),
        });
      }

      match data.type_info.type_.as_ref() {
        RustType::Box(_) => {
          if !matches!(
            data.proto_field,
            ProtoField::Single(ProtoType::Message { is_boxed: true, .. })
          ) {
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
        variant,
        "Cannot use `ignore` in direct impls. Use a proxied impl instead"
      );
    }
  }

  sort_and_check_invalid_tags(&mut manually_set_tags, &ReservedNumbers::default())?;

  for (variant, field_attrs) in variants.iter_mut().zip(fields_data.iter()) {
    let Some(tag) = field_attrs.tag else {
      bail!(variant, "Tags in oneofs must be set manually");
    };

    let prost_attr = field_attrs.proto_field.as_prost_attr(tag);
    variant.attrs.push(prost_attr);

    // We change the type in direct impls as well,
    // mostly just to be able to use the real enum names
    // as opposed to just an opaque `i32`
    let prost_compatible_type = field_attrs.proto_field.output_proto_type();
    *variant.type_mut()? = prost_compatible_type;
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

  let wrapped_items = wrap_with_imports(vec![oneof_schema_impl, validator_impl]);

  let output = quote! {
    #wrapped_items
    #consistency_checks_impl
  };

  Ok(output)
}
