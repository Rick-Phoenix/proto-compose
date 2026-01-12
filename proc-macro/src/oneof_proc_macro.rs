use crate::*;

pub struct OneofCtx<'a> {
  pub oneof_attrs: &'a OneofAttrs,
  pub orig_enum_ident: &'a Ident,
  pub shadow_enum_ident: Option<&'a Ident>,
  pub variants: Vec<FieldDataKind>,
  pub tags: Vec<ParsedNum>,
}

impl<'a> OneofCtx<'a> {
  pub fn proto_enum_ident(&'a self) -> &'a Ident {
    self
      .shadow_enum_ident
      .as_ref()
      .unwrap_or(&self.orig_enum_ident)
  }
}

pub fn process_oneof_proc_macro(mut item: ItemEnum, macro_attrs: TokenStream2) -> TokenStream2 {
  let mut errors: Vec<Error> = Vec::new();
  let mut output = TokenStream2::new();

  let macro_attrs =
    OneofMacroAttrs::parse(macro_attrs).unwrap_or_default_and_push_error(&mut errors);

  let oneof_attrs = process_oneof_attrs(&item.ident, macro_attrs, &item.attrs)
    .unwrap_or_default_and_push_error(&mut errors);

  let is_proxied = oneof_attrs.is_proxied;

  let mut shadow_enum = is_proxied.then(|| create_shadow_enum(&item));

  let FieldsCtx {
    mut fields_data,
    mut manually_set_tags,
  } = extract_fields_data(
    item.variants.len(),
    item
      .variants
      .iter_mut()
      .map(|v| FieldOrVariant::Variant(v)),
  )
  .unwrap_or_default_and_push_error(&mut errors);

  if let Err(e) = sort_and_check_invalid_tags(&mut manually_set_tags, &ReservedNumbers::default()) {
    errors.push(e);
  }

  let impl_kind = if is_proxied {
    ImplKind::Shadow
  } else {
    ImplKind::Direct
  };

  let enum_to_process = shadow_enum.as_mut().unwrap_or(&mut item);

  second_processing(
    impl_kind,
    enum_to_process
      .variants
      .iter_mut()
      .map(|v| FieldOrVariant::Variant(v)),
    &mut fields_data,
    None,
    ContainerAttrs::Oneof(&oneof_attrs),
    ItemKind::Oneof,
  )
  .unwrap_or_default_and_push_error(&mut errors);

  // prost::Oneof already implements Debug and Default
  let proto_derives = if !errors.is_empty() {
    FallbackImpls {
      orig_ident: &item.ident,
      shadow_ident: shadow_enum.as_ref().map(|se| &se.ident),
      kind: ItemKind::Oneof,
    }
    .fallback_derive_impls()
  } else if cfg!(feature = "cel") {
    quote! {
      #[allow(clippy::derive_partial_eq_without_eq)]
      #[derive(::prost::Oneof, Clone, PartialEq, ::prelude::CelOneof)]
    }
  } else {
    quote! {
      #[allow(clippy::derive_partial_eq_without_eq)]
      #[derive(::prost::Oneof, Clone, PartialEq)]
    }
  };

  if !errors.is_empty() {
    fields_data.clear();
  }

  if let Some(shadow_enum) = &mut shadow_enum {
    // We strip away the ignored variants from the shadow enum
    let shadow_variants = std::mem::take(&mut shadow_enum.variants);
    shadow_enum.variants = shadow_variants
      .into_iter()
      .zip(fields_data.iter())
      .filter_map(|(variant, data)| matches!(data, FieldDataKind::Normal(_)).then_some(variant))
      .collect();

    let shadow_enum_derives = oneof_attrs
      .shadow_derives
      .as_ref()
      .map(|list| quote! { #[#list] });

    let conversions = ProtoConversionImpl {
      source_ident: item.ident.clone(),
      target_ident: shadow_enum.ident.clone(),
      kind: ItemKind::Oneof,
      into_proto: ConversionData::new(oneof_attrs.into_proto.as_ref()),
      from_proto: ConversionData::new(oneof_attrs.from_proto.as_ref()),
    }
    .generate_conversion_impls(&fields_data);

    output.extend(quote! {
      #[derive(::prelude::macros::Oneof)]
      #item

      #proto_derives
      #shadow_enum_derives
      #[allow(clippy::use_self)]
      #shadow_enum

      #conversions
    });
  } else {
    output.extend(quote! {
      #proto_derives
      #[derive(::prelude::macros::Oneof)]
      #item
    });
  }

  let oneof_ctx = OneofCtx {
    oneof_attrs: &oneof_attrs,
    orig_enum_ident: &item.ident,
    shadow_enum_ident: shadow_enum.as_ref().map(|se| &se.ident),
    variants: fields_data,
    tags: manually_set_tags,
  };

  let consistency_checks_impl = oneof_ctx.generate_consistency_checks();
  let validator_impl = oneof_ctx.generate_validator();
  let schema_impls = oneof_ctx.generate_schema_impl();

  let wrapped_items = wrap_with_imports(&[schema_impls, validator_impl]);

  output.extend(wrapped_items);
  output.extend(consistency_checks_impl);

  output.extend(errors.iter().map(|e| e.to_compile_error()));

  output
}
