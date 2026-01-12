use crate::*;

pub fn message_proc_macro(mut item: ItemStruct, macro_attrs: TokenStream2) -> TokenStream2 {
  let mut errors: Vec<Error> = Vec::new();
  let mut output = TokenStream2::new();

  let macro_args =
    MessageMacroArgs::parse(macro_attrs).unwrap_or_default_and_push_error(&mut errors);

  let message_attrs = process_message_attrs(&item.ident, macro_args, &item.attrs)
    .unwrap_or_default_and_push_error(&mut errors);

  let is_proxied = message_attrs.is_proxied;

  let mut proxy_struct = is_proxied.then(|| create_shadow_struct(&item));

  let FieldsCtx {
    mut fields_data,
    mut manually_set_tags,
  } = extract_fields_data(&mut item.fields).unwrap_or_default_and_push_error(&mut errors);

  let used_ranges =
    build_unavailable_ranges(&message_attrs.reserved_numbers, &mut manually_set_tags)
      .unwrap_or_default_and_push_error(&mut errors);

  let mut tag_allocator = TagAllocator::new(&used_ranges);

  let impl_kind = if is_proxied {
    ImplKind::Shadow
  } else {
    ImplKind::Direct
  };

  second_processing(
    impl_kind,
    proxy_struct
      .as_mut()
      .map(|ps| &mut ps.fields)
      .unwrap_or(&mut item.fields),
    &mut fields_data,
    &mut tag_allocator,
    &message_attrs,
  )
  .unwrap_or_default_and_push_error(&mut errors);

  // prost::Message already implements Debug and Default
  let proto_derives = if !errors.is_empty() {
    FallbackImpls {
      orig_ident: &item.ident,
      shadow_ident: proxy_struct.as_ref().map(|ps| &ps.ident),
      kind: InputItemKind::Message,
    }
    .fallback_derive_impls()
  } else if cfg!(feature = "cel") {
    quote! {
      #[allow(clippy::derive_partial_eq_without_eq)]
      #[derive(::prost::Message, Clone, PartialEq, ::prelude::CelValue)]
    }
  } else {
    quote! {
      #[allow(clippy::derive_partial_eq_without_eq)]
      #[derive(::prost::Message, Clone, PartialEq)]
    }
  };

  if !errors.is_empty() {
    // This will trigger all of the fallback impls that expand to unimplemented!
    fields_data.clear();
  }

  if let Some(proxy_struct) = &mut proxy_struct {
    if let Fields::Named(fields) = &mut proxy_struct.fields {
      let old_fields = std::mem::take(&mut fields.named);

      fields.named = old_fields
        .into_iter()
        .zip(fields_data.iter())
        .filter_map(|(field, data)| matches!(data, FieldDataKind::Normal(_)).then_some(field))
        .collect();
    }

    let shadow_struct_derives = message_attrs
      .shadow_derives
      .as_ref()
      .map(|list| quote! { #[#list] });

    let conversions = ProtoConversionImpl {
      source_ident: item.ident.clone(),
      target_ident: proxy_struct.ident.clone(),
      kind: InputItemKind::Message,
      into_proto: ConversionData::new(message_attrs.into_proto.as_ref()),
      from_proto: ConversionData::new(message_attrs.from_proto.as_ref()),
    }
    .generate_conversion_impls(&fields_data);

    output.extend(quote! {
      #[derive(::prelude::macros::Message)]
      #item

      #proto_derives
      #shadow_struct_derives
      #proxy_struct

      #conversions
    });
  } else {
    output.extend(quote! {
      #proto_derives
      #[derive(::prelude::macros::Message)]
      #item
    });
  }

  // Consistency, validator, schema
  let message_ctx = MessageCtx {
    orig_struct_ident: item.ident.clone(),
    shadow_struct_ident: proxy_struct.as_ref().map(|ps| ps.ident.clone()),
    fields_data,
    message_attrs: &message_attrs,
  };

  let consistency_checks_impl = message_ctx.generate_consistency_checks();
  let validator_impl = message_ctx.generate_validator();
  let schema_impls = message_ctx.generate_schema_impls();

  let wrapped_items = wrap_with_imports(&[schema_impls, validator_impl]);

  output.extend(wrapped_items);
  output.extend(consistency_checks_impl);

  output.extend(errors.iter().map(|e| e.to_compile_error()));

  output
}

pub struct MessageCtx<'a> {
  pub orig_struct_ident: Ident,
  pub shadow_struct_ident: Option<Ident>,
  pub fields_data: Vec<FieldDataKind>,
  pub message_attrs: &'a MessageAttrs,
}

impl<'a> MessageCtx<'a> {
  pub fn proto_struct_ident(&'a self) -> &'a Ident {
    self
      .shadow_struct_ident
      .as_ref()
      .unwrap_or(&self.orig_struct_ident)
  }
}

#[derive(Default)]
pub struct FieldsCtx {
  pub fields_data: Vec<FieldDataKind>,
  pub manually_set_tags: Vec<ParsedNum>,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ImplKind {
  #[default]
  Direct,
  Shadow,
}

impl ImplKind {
  /// Returns `true` if the impl kind is [`Direct`].
  ///
  /// [`Direct`]: ImplKind::Direct
  #[must_use]
  pub fn is_direct(&self) -> bool {
    matches!(self, Self::Direct)
  }

  /// Returns `true` if the impl kind is [`Shadow`].
  ///
  /// [`Shadow`]: ImplKind::Shadow
  #[must_use]
  pub fn is_shadow(&self) -> bool {
    matches!(self, Self::Shadow)
  }
}

pub fn second_processing(
  impl_kind: ImplKind,
  fields: &mut Fields,
  fields_data: &mut [FieldDataKind],
  tag_allocator: &mut TagAllocator,
  message_attrs: &MessageAttrs,
) -> syn::Result<()> {
  for (dst_field, field_data) in fields.iter_mut().zip(fields_data.iter_mut()) {
    // Skipping ignored fields
    let FieldDataKind::Normal(field_data) = field_data else {
      if impl_kind.is_shadow() {
        continue;
      } else {
        bail!(
          dst_field.require_ident()?,
          "Cannot ignore fields in a direct impl"
        );
      }
    };

    if !field_data.proto_field.is_oneof() && field_data.tag.is_none() {
      let new_tag = tag_allocator.next_tag(field_data.span)?;

      field_data.tag = Some(ParsedNum {
        num: new_tag,
        span: field_data.span,
      });
    };

    let prost_attr = field_data.as_prost_attr();
    dst_field.attrs.push(prost_attr);

    // Proxy impl errors
    if impl_kind.is_shadow() {
      let prost_compatible_type = field_data.output_proto_type(false);
      dst_field.ty = prost_compatible_type;

      if let ProtoField::Oneof(OneofInfo { default: false, .. }) = &field_data.proto_field
        && !field_data.type_info.is_option()
        && !field_data.has_custom_conversions()
        && !message_attrs.has_custom_conversions()
      {
        bail!(
          &field_data.type_info,
          "A oneof must be wrapped in `Option` unless a custom to/from proto implementation is provided or the `default` attribute is used"
        );
      }

      if let Some(ProtoType::Message(MessageInfo { default: false, .. })) =
        field_data.proto_field.inner()
        && !field_data.type_info.is_option()
        && !field_data.has_custom_conversions()
        && !message_attrs.has_custom_conversions()
      {
        bail!(
          &field_data.type_info,
          "A message must be wrapped in `Option` unless a custom to/from proto implementation is provided or the `default` attribute is used"
        );
      }
      // Direct impl errors
    } else {
      if field_data.proto_field.is_enum() && !field_data.type_info.inner().is_int() {
        bail!(
          &field_data.type_info,
          "Enums must use `i32` in direct impls"
        )
      }

      if field_data.proto_field.is_oneof() && !field_data.type_info.is_option() {
        bail!(
          &field_data.type_info,
          "Oneofs must be wrapped in `Option` in a direct impl"
        )
      }

      match field_data.type_info.type_.as_ref() {
        RustType::Box(inner) if field_data.proto_field.is_message() => {
          bail!(inner, "Boxed messages must be optional in a direct impl")
        }
        RustType::Other(inner) => {
          if field_data.proto_field.is_message() {
            bail!(
              &inner.path,
              "Messages must be wrapped in Option in direct impls"
            );
          }
        }
        _ => {}
      };
    }
  }

  Ok(())
}

pub fn extract_fields_data(fields: &mut Fields) -> syn::Result<FieldsCtx> {
  let mut fields_data: Vec<FieldDataKind> = Vec::with_capacity(fields.len());
  let mut manually_set_tags: Vec<ParsedNum> = Vec::with_capacity(fields.len());

  for field in fields.iter_mut() {
    let field_data_kind = process_field_data(FieldOrVariant::Field(field))?;

    if let FieldDataKind::Normal(data) = &field_data_kind {
      if let Some(tag) = data.tag {
        manually_set_tags.push(tag);
      } else if let ProtoField::Oneof(OneofInfo { tags, .. }) = &data.proto_field {
        manually_set_tags.extend(tags);
      }
    }

    fields_data.push(field_data_kind);
  }

  manually_set_tags.shrink_to_fit();

  Ok(FieldsCtx {
    fields_data,
    manually_set_tags,
  })
}

// pub fn message_macro_shadow(
//   orig_struct: &mut ItemStruct,
//   shadow_struct: &mut ItemStruct,
//   message_attrs: &MessageAttrs,
// ) -> Result<TokenStream2, Error> {
//   let orig_struct_ident = &orig_struct.ident;
//   let shadow_struct_ident = &shadow_struct.ident;
//
//   let mut ignored_fields: Vec<Ident> = Vec::new();
//
//   let mut proto_conversion_data = ProtoConversionImpl {
//     source_ident: orig_struct_ident,
//     target_ident: shadow_struct_ident,
//     kind: InputItemKind::Message,
//     into_proto: ConversionData::new(message_attrs.into_proto.as_ref()),
//     from_proto: ConversionData::new(message_attrs.from_proto.as_ref()),
//   };
//
//   let mut fields_data: Vec<FieldDataKind> = Vec::new();
//   let mut manually_set_tags: Vec<ParsedNum> = Vec::new();
//
//   for field in orig_struct.fields.iter_mut() {
//     let field_data_kind = process_field_data(FieldOrVariant::Field(field))?;
//
//     proto_conversion_data.handle_field_conversions(&field_data_kind);
//
//     match &field_data_kind {
//       FieldDataKind::Ignored { .. } => {
//         ignored_fields.push(field.require_ident()?.clone());
//       }
//
//       FieldDataKind::Normal(data) => {
//         if let Some(tag) = data.tag {
//           manually_set_tags.push(tag);
//         } else if let ProtoField::Oneof(OneofInfo { tags, .. }) = &data.proto_field {
//           for tag in tags.iter().copied() {
//             manually_set_tags.push(tag);
//           }
//         }
//       }
//     };
//
//     fields_data.push(field_data_kind);
//   }
//
//   let used_ranges =
//     build_unavailable_ranges(&message_attrs.reserved_numbers, &mut manually_set_tags)?;
//
//   let mut tag_allocator = TagAllocator::new(&used_ranges);
//
//   for (dst_field, field_data) in shadow_struct
//     .fields
//     .iter_mut()
//     .zip(fields_data.iter_mut())
//   {
//     // Skipping ignored fields
//     let FieldDataKind::Normal(field_data) = field_data else {
//       continue;
//     };
//
//     if let ProtoField::Oneof(OneofInfo { default: false, .. }) = &field_data.proto_field
//       && !field_data.type_info.is_option()
//       && !field_data.has_custom_conversions()
//       && !proto_conversion_data.has_custom_impls()
//     {
//       bail!(
//         &field_data.type_info,
//         "A oneof must be wrapped in `Option` unless a custom to/from proto implementation is provided or the `default` attribute is used"
//       );
//     }
//
//     if let Some(ProtoType::Message(MessageInfo { default: false, .. })) =
//       field_data.proto_field.inner()
//       && !field_data.type_info.is_option()
//       && !field_data.has_custom_conversions()
//       && !proto_conversion_data.has_custom_impls()
//     {
//       bail!(
//         &field_data.type_info,
//         "A message must be wrapped in `Option` unless a custom to/from proto implementation is provided or the `default` attribute is used"
//       );
//     }
//
//     if !field_data.proto_field.is_oneof() && field_data.tag.is_none() {
//       let new_tag = tag_allocator.next_tag(field_data.span)?;
//
//       field_data.tag = Some(ParsedNum {
//         num: new_tag,
//         span: field_data.span,
//       });
//     };
//
//     let prost_attr = field_data.as_prost_attr();
//     dst_field.attrs.push(prost_attr);
//
//     let prost_compatible_type = field_data.output_proto_type(false);
//     dst_field.ty = prost_compatible_type;
//   }
//
//   // We strip away the ignored fields from the shadow struct
//   if let Fields::Named(fields) = &mut shadow_struct.fields {
//     let old_fields = std::mem::take(&mut fields.named);
//
//     fields.named = old_fields
//       .into_iter()
//       .filter(|f| !ignored_fields.contains(f.ident.as_ref().unwrap()))
//       .collect();
//   }
//
//   // Into/From proto impls
//   let proto_conversion_impls = proto_conversion_data.generate_conversion_impls();
//
//   let non_ignored_fields: Vec<&FieldData> = fields_data
//     .iter()
//     .filter_map(|f| f.as_normal())
//     .collect();
//
//   let message_ctx = MessageCtx {
//     orig_struct_ident,
//     shadow_struct_ident: Some(shadow_struct_ident),
//     non_ignored_fields,
//     message_attrs,
//   };
//
//   let consistency_checks_impl = message_ctx.generate_consistency_checks();
//   let validator_impl = message_ctx.generate_validator();
//   let schema_impls = message_ctx.generate_schema_impls();
//
//   let wrapped_items = wrap_with_imports(&[schema_impls, proto_conversion_impls, validator_impl]);
//
//   Ok(quote! {
//     #wrapped_items
//     #consistency_checks_impl
//   })
// }
//
// pub fn message_macro_direct(
//   item: &mut ItemStruct,
//   message_attrs: &MessageAttrs,
// ) -> Result<TokenStream2, Error> {
//   let mut fields_data: Vec<FieldData> = Vec::new();
//   let mut manually_set_tags: Vec<ParsedNum> = Vec::new();
//
//   for field in item.fields.iter_mut() {
//     let field_data_kind = process_field_data(FieldOrVariant::Field(field))?;
//
//     if let FieldDataKind::Normal(data) = field_data_kind {
//       if data.proto_field.is_enum() && !data.type_info.inner().is_int() {
//         bail!(&data.type_info, "Enums must use `i32` in direct impls")
//       }
//
//       if data.proto_field.is_oneof() && !data.type_info.is_option() {
//         bail!(
//           &data.type_info,
//           "Oneofs must be wrapped in `Option` in a direct impl"
//         )
//       }
//
//       match data.type_info.type_.as_ref() {
//         RustType::Box(inner) if data.proto_field.is_message() => {
//           bail!(inner, "Boxed messages must be optional in a direct impl")
//         }
//         RustType::Other(inner) => {
//           if data.proto_field.is_message() {
//             bail!(
//               &inner.path,
//               "Messages must be wrapped in Option in direct impls"
//             );
//           }
//         }
//         _ => {}
//       };
//
//       if let Some(tag) = data.tag {
//         manually_set_tags.push(tag);
//       } else if let ProtoField::Oneof(OneofInfo { tags, .. }) = &data.proto_field {
//         for tag in tags.iter().copied() {
//           manually_set_tags.push(tag);
//         }
//       }
//
//       fields_data.push(data);
//     } else {
//       bail!(
//         field.require_ident()?,
//         "Cannot use `ignore` in a direct impl. Use a proxied impl instead"
//       );
//     }
//   }
//
//   let used_ranges =
//     build_unavailable_ranges(&message_attrs.reserved_numbers, &mut manually_set_tags)?;
//
//   let mut tag_allocator = TagAllocator::new(&used_ranges);
//
//   for (field, field_data) in item.fields.iter_mut().zip(fields_data.iter_mut()) {
//     if !field_data.proto_field.is_oneof() && field_data.tag.is_none() {
//       let new_tag = tag_allocator.next_tag(field_data.span)?;
//
//       field_data.tag = Some(ParsedNum {
//         num: new_tag,
//         span: field_data.span,
//       });
//     };
//
//     let prost_attr = field_data.as_prost_attr();
//     field.attrs.push(prost_attr);
//   }
//
//   let message_ctx = MessageCtx {
//     orig_struct_ident: &item.ident,
//     shadow_struct_ident: None,
//     non_ignored_fields: fields_data,
//     message_attrs,
//   };
//
//   let consistency_checks_impl = message_ctx.generate_consistency_checks();
//   let schema_impls = message_ctx.generate_schema_impls();
//   let validator_impl = message_ctx.generate_validator();
//
//   let wrapped_items = wrap_with_imports(&[schema_impls, validator_impl]);
//
//   let output_tokens = quote! {
//     #wrapped_items
//     #consistency_checks_impl
//   };
//
//   Ok(output_tokens)
// }
