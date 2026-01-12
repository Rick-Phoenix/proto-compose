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
    InputItemKind::Oneof,
  )
  .unwrap_or_default_and_push_error(&mut errors);

  // prost::Oneof already implements Debug and Default
  let proto_derives = if !errors.is_empty() {
    FallbackImpls {
      orig_ident: &item.ident,
      shadow_ident: shadow_enum.as_ref().map(|se| &se.ident),
      kind: InputItemKind::Oneof,
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
      kind: InputItemKind::Oneof,
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

  // Consistency, validator, schema
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

// pub(crate) fn oneof_shadow_proc_macro(
//   item: &mut ItemEnum,
//   shadow_enum: &mut ItemEnum,
//   oneof_attrs: &OneofAttrs,
// ) -> Result<TokenStream2, Error> {
//   let orig_enum_ident = &item.ident;
//   let shadow_enum_ident = &shadow_enum.ident;
//
//   let mut ignored_variants: Vec<Ident> = Vec::new();
//
//   let mut proto_conversion_data = ProtoConversionImpl {
//     source_ident: orig_enum_ident.clone(),
//     target_ident: shadow_enum_ident.clone(),
//     kind: InputItemKind::Oneof,
//     into_proto: ConversionData::new(oneof_attrs.into_proto.as_ref()),
//     from_proto: ConversionData::new(oneof_attrs.from_proto.as_ref()),
//   };
//
//   let mut manually_set_tags: Vec<ParsedNum> = Vec::new();
//   let mut fields_data: Vec<FieldDataKind> = Vec::new();
//
//   for src_variant in item.variants.iter_mut() {
//     let field_data_kind = process_field_data(FieldOrVariant::Variant(src_variant))?;
//     proto_conversion_data.handle_field_conversions(&field_data_kind);
//
//     match &field_data_kind {
//       FieldDataKind::Ignored { ident, .. } => ignored_variants.push(ident.clone()),
//       FieldDataKind::Normal(data) => {
//         if let Some(tag) = data.tag {
//           manually_set_tags.push(tag);
//         }
//       }
//     };
//
//     fields_data.push(field_data_kind);
//   }
//
//   sort_and_check_invalid_tags(&mut manually_set_tags, &ReservedNumbers::default())?;
//
//   for (dst_variant, field_attrs) in shadow_enum
//     .variants
//     .iter_mut()
//     .zip(fields_data.iter())
//   {
//     // Skipping ignored variants
//     let FieldDataKind::Normal(field_attrs) = field_attrs else {
//       continue;
//     };
//
//     if field_attrs.tag.is_none() {
//       bail!(dst_variant.ident, "Tags in oneofs must be set manually");
//     };
//
//     let prost_attr = field_attrs.as_prost_attr();
//     dst_variant.attrs.push(prost_attr);
//
//     let prost_compatible_type = field_attrs.output_proto_type(true);
//     *dst_variant.type_mut()? = prost_compatible_type;
//   }
//
//   // We strip away the ignored variants from the shadow enum
//   let shadow_variants = std::mem::take(&mut shadow_enum.variants);
//   shadow_enum.variants = shadow_variants
//     .into_iter()
//     .filter(|var| !ignored_variants.contains(&var.ident))
//     .collect();
//
//   let non_ignored_variants: Vec<&FieldData> = fields_data
//     .iter()
//     .filter_map(|f| f.as_normal())
//     .collect();
//
//   let proto_conversion_impls = proto_conversion_data.generate_conversion_impls();
//
//   let oneof_ctx = OneofCtx {
//     oneof_attrs,
//     orig_enum_ident,
//     shadow_enum_ident: Some(shadow_enum_ident),
//     variants: non_ignored_variants,
//     tags: manually_set_tags,
//   };
//
//   let oneof_schema_impl = oneof_ctx.generate_schema_impl();
//   let consistency_checks_impl = oneof_ctx.generate_consistency_checks();
//   let validator_impl = oneof_ctx.generate_validator();
//
//   let wrapped_items =
//     wrap_with_imports(&[oneof_schema_impl, proto_conversion_impls, validator_impl]);
//
//   Ok(quote! {
//     #wrapped_items
//     #consistency_checks_impl
//   })
// }
//
// pub(crate) fn oneof_direct_proc_macro(
//   item: &mut ItemEnum,
//   oneof_attrs: &OneofAttrs,
// ) -> Result<TokenStream2, Error> {
//   let ItemEnum { variants, .. } = item;
//
//   let mut manually_set_tags: Vec<ParsedNum> = Vec::new();
//   let mut fields_data: Vec<FieldData> = Vec::new();
//
//   for variant in variants.iter_mut() {
//     let field_attrs = process_field_data(FieldOrVariant::Variant(variant))?;
//     let variant_type = variant.type_()?;
//
//     if let FieldDataKind::Normal(data) = field_attrs {
//       if let Some(tag) = data.tag {
//         manually_set_tags.push(tag);
//       }
//
//       if data.proto_field.is_enum() && !data.type_info.inner().is_int() {
//         bail!(&data.type_info, "Enums must use `i32` in direct impls")
//       }
//
//       match data.type_info.type_.as_ref() {
//         RustType::Box(_) => {
//           if !data.proto_field.is_boxed_message() {
//             bail!(
//               variant_type,
//               "Box can only be used for messages in a native prost oneof"
//             );
//           }
//         }
//
//         // For unknown types such as messages
//         RustType::Other(_) => {}
//
//         _ => {
//           if !data.type_info.type_.is_primitive() && !data.type_info.type_.is_bytes() {
//             bail!(
//               variant_type,
//               "Unsupported Oneof variant type. If you want to use a custom type, you must use a proxied oneof with custom conversions"
//             )
//           }
//         }
//       };
//
//       fields_data.push(data);
//     } else {
//       bail!(
//         variant.ident,
//         "Cannot use `ignore` in direct impls. Use a proxied impl instead"
//       );
//     }
//   }
//
//   sort_and_check_invalid_tags(&mut manually_set_tags, &ReservedNumbers::default())?;
//
//   for (variant, field_attrs) in variants.iter_mut().zip(fields_data.iter()) {
//     if field_attrs.tag.is_none() {
//       bail!(variant.ident, "Tags in oneofs must be set manually");
//     };
//
//     let prost_attr = field_attrs.as_prost_attr();
//     variant.attrs.push(prost_attr);
//   }
//
//   let oneof_ctx = OneofCtx {
//     oneof_attrs,
//     orig_enum_ident: &item.ident,
//     shadow_enum_ident: None,
//     variants: fields_data,
//     tags: manually_set_tags,
//   };
//
//   let oneof_schema_impl = oneof_ctx.generate_schema_impl();
//   let consistency_checks_impl = oneof_ctx.generate_consistency_checks();
//   let validator_impl = oneof_ctx.generate_validator();
//
//   let wrapped_items = wrap_with_imports(&[oneof_schema_impl, validator_impl]);
//
//   let output = quote! {
//     #wrapped_items
//     #consistency_checks_impl
//   };
//
//   Ok(output)
// }
