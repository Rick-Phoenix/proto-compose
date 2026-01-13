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

  let macro_attrs =
    OneofMacroAttrs::parse(macro_attrs).unwrap_or_default_and_push_error(&mut errors);

  let oneof_attrs = process_oneof_attrs(&item.ident, macro_attrs, &item.attrs)
    .unwrap_or_default_and_push_error(&mut errors);

  let is_proxied = oneof_attrs.is_proxied;

  let mut proto_enum = is_proxied.then(|| create_shadow_enum(&item));

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
    ImplKind::Proxied
  } else {
    ImplKind::Direct
  };

  let enum_to_process = proto_enum.as_mut().unwrap_or(&mut item);

  ProcessFieldsData {
    impl_kind,
    fields: enum_to_process
      .variants
      .iter_mut()
      .map(|v| FieldOrVariant::Variant(v)),
    fields_data: &mut fields_data,
    // Tags in oneofs must be set manually
    tag_allocator: None,
    container_attrs: ContainerAttrs::Oneof(&oneof_attrs),
    item_kind: ItemKind::Oneof,
  }
  .process_fields_data()
  .unwrap_or_default_and_push_error(&mut errors);

  let proto_derives = if !errors.is_empty() {
    FallbackImpls {
      orig_ident: &item.ident,
      proto_ident: proto_enum.as_ref().map(|se| &se.ident),
      kind: ItemKind::Oneof,
    }
    .fallback_derive_impls()
  } else if cfg!(feature = "cel") {
    // prost::Oneof already implements Debug and Default
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
    // This will trigger all of the fallback impls that expand to unimplemented!
    fields_data.clear();
  }

  let main_enum_tokens = if let Some(proto_enum) = &mut proto_enum {
    // We strip away the ignored variants from the shadow enum
    let shadow_variants = std::mem::take(&mut proto_enum.variants);
    proto_enum.variants = shadow_variants
      .into_iter()
      .zip(fields_data.iter())
      .filter_map(|(variant, data)| matches!(data, FieldDataKind::Normal(_)).then_some(variant))
      .collect();

    let shadow_enum_derives = oneof_attrs
      .shadow_derives
      .as_ref()
      .map(|list| quote! { #[#list] });

    let conversions = ProtoConversions {
      proxy_ident: &item.ident,
      proto_ident: &proto_enum.ident,
      kind: ItemKind::Oneof,
      container_attrs: ContainerAttrs::Oneof(&oneof_attrs),
      fields: &fields_data,
    }
    .generate_proto_conversions();

    quote! {
      #[derive(::prelude::macros::Oneof)]
      #item

      #proto_derives
      #shadow_enum_derives
      #[allow(clippy::use_self)]
      #proto_enum

      #conversions
    }
  } else {
    quote! {
      #proto_derives
      #[derive(::prelude::macros::Oneof)]
      #item
    }
  };

  let oneof_ctx = OneofCtx {
    oneof_attrs: &oneof_attrs,
    orig_enum_ident: &item.ident,
    shadow_enum_ident: proto_enum.as_ref().map(|se| &se.ident),
    variants: fields_data,
    tags: manually_set_tags,
  };

  let consistency_checks_impl = errors
    .is_empty()
    .then(|| oneof_ctx.generate_consistency_checks());
  let validator_impl = oneof_ctx.generate_validator();
  let schema_impls = oneof_ctx.generate_schema_impl();

  let wrapped_items = wrap_with_imports(&[schema_impls, validator_impl]);

  let errors = errors.iter().map(|e| e.to_compile_error());

  quote! {
    #main_enum_tokens
    #wrapped_items
    #consistency_checks_impl
    #(#errors)*
  }
}
