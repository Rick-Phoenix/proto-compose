use crate::*;

pub struct MessageCtx<'a, T: Borrow<FieldData>> {
  pub orig_struct_ident: &'a Ident,
  pub shadow_struct_ident: Option<&'a Ident>,
  pub non_ignored_fields: Vec<T>,
  pub message_attrs: &'a MessageAttrs,
}

impl<'a, T: Borrow<FieldData>> MessageCtx<'a, T> {
  pub fn proto_struct_ident(&self) -> &'a Ident {
    self
      .shadow_struct_ident
      .unwrap_or(self.orig_struct_ident)
  }
}

pub fn message_proc_macro(mut item: ItemStruct, macro_attrs: TokenStream2) -> TokenStream2 {
  let message_attrs = match process_message_attrs(&item.ident, macro_attrs, &item.attrs) {
    Ok(attrs) => attrs,
    Err(e) => {
      let err = e.into_compile_error();

      return quote! {
        #item
        #err
      };
    }
  };

  // prost::Message already implements Debug and Default
  let mut proto_derives = if cfg!(feature = "cel") {
    quote! {
      #[derive(::prost::Message, Clone, PartialEq, ::prelude::CelValue)]
      #[cel(cel_crate = ::prelude::cel, proto_types_crate = ::prelude::proto_types)]
    }
  } else {
    quote! { #[derive(::prost::Message, Clone, PartialEq)] }
  };

  if message_attrs.is_proxied {
    let mut shadow_struct = create_shadow_struct(&item);

    let impls = match message_macro_shadow(&mut item, &mut shadow_struct, &message_attrs) {
      Ok(impls) => impls,
      Err(e) => {
        proto_derives = TokenStream2::new();

        FallbackImpls {
          error: e,
          orig_ident: &item.ident,
          shadow_ident: Some(&shadow_struct.ident),
          kind: InputItemKind::Message,
        }
        .generate_fallback_impls()
      }
    };

    let shadow_struct_derives = message_attrs
      .shadow_derives
      .map(|list| quote! { #[#list] });

    quote! {
      #[allow(clippy::derive_partial_eq_without_eq)]
      #[derive(::prelude::macros::Message)]
      #item

      #[allow(clippy::derive_partial_eq_without_eq)]
      #proto_derives
      #shadow_struct_derives
      #shadow_struct

      #impls
    }
  } else {
    let impls = match message_macro_direct(&mut item, &message_attrs) {
      Ok(impls) => impls,
      Err(e) => {
        proto_derives = TokenStream2::new();

        FallbackImpls {
          error: e,
          orig_ident: &item.ident,
          shadow_ident: None,
          kind: InputItemKind::Message,
        }
        .generate_fallback_impls()
      }
    };

    quote! {
      #[allow(clippy::derive_partial_eq_without_eq)]
      #[derive(::prelude::macros::Message)]
      #proto_derives
      #item

      #impls
    }
  }
}

pub fn message_macro_shadow(
  orig_struct: &mut ItemStruct,
  shadow_struct: &mut ItemStruct,
  message_attrs: &MessageAttrs,
) -> Result<TokenStream2, Error> {
  let orig_struct_ident = &orig_struct.ident;
  let shadow_struct_ident = &shadow_struct.ident;

  let mut ignored_fields: Vec<Ident> = Vec::new();

  let mut proto_conversion_data = ProtoConversionImpl {
    source_ident: orig_struct_ident,
    target_ident: shadow_struct_ident,
    kind: InputItemKind::Message,
    into_proto: ConversionData::new(message_attrs.into_proto.as_ref()),
    from_proto: ConversionData::new(message_attrs.from_proto.as_ref()),
  };

  let mut fields_data: Vec<FieldDataKind> = Vec::new();
  let mut manually_set_tags: Vec<ParsedNum> = Vec::new();

  for field in orig_struct.fields.iter_mut() {
    let field_data_kind = process_field_data(FieldOrVariant::Field(field))?;

    proto_conversion_data.handle_field_conversions(&field_data_kind);

    match &field_data_kind {
      FieldDataKind::Ignored { .. } => {
        ignored_fields.push(field.require_ident()?.clone());
      }

      FieldDataKind::Normal(data) => {
        if let Some(tag) = data.tag {
          manually_set_tags.push(tag);
        } else if let ProtoField::Oneof(OneofInfo { tags, .. }) = &data.proto_field {
          for tag in tags.iter().copied() {
            manually_set_tags.push(tag);
          }
        }
      }
    };

    fields_data.push(field_data_kind);
  }

  let used_ranges =
    build_unavailable_ranges(&message_attrs.reserved_numbers, &mut manually_set_tags)?;

  let mut tag_allocator = TagAllocator::new(&used_ranges);

  for (dst_field, field_data) in shadow_struct
    .fields
    .iter_mut()
    .zip(fields_data.iter_mut())
  {
    // Skipping ignored fields
    let FieldDataKind::Normal(field_data) = field_data else {
      continue;
    };

    if let ProtoField::Oneof(OneofInfo { default: false, .. }) = &field_data.proto_field
      && !field_data.type_info.is_option()
      && !field_data.has_custom_conversions()
      && !proto_conversion_data.has_custom_impls()
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
      && !proto_conversion_data.has_custom_impls()
    {
      bail!(
        &field_data.type_info,
        "A message must be wrapped in `Option` unless a custom to/from proto implementation is provided or the `default` attribute is used"
      );
    }

    if !field_data.proto_field.is_oneof() && field_data.tag.is_none() {
      let new_tag = tag_allocator.next_tag(field_data.span)?;

      field_data.tag = Some(ParsedNum {
        num: new_tag,
        span: field_data.span,
      });
    };

    let prost_attr = field_data.as_prost_attr();
    dst_field.attrs.push(prost_attr);

    let prost_compatible_type = field_data.output_proto_type(false);
    dst_field.ty = prost_compatible_type;
  }

  // We strip away the ignored fields from the shadow struct
  if let Fields::Named(fields) = &mut shadow_struct.fields {
    let old_fields = std::mem::take(&mut fields.named);

    fields.named = old_fields
      .into_iter()
      .filter(|f| !ignored_fields.contains(f.ident.as_ref().unwrap()))
      .collect();
  }

  // Into/From proto impls
  let proto_conversion_impls = proto_conversion_data.generate_conversion_impls();

  let non_ignored_fields: Vec<&FieldData> = fields_data
    .iter()
    .filter_map(|f| f.as_normal())
    .collect();

  let message_ctx = MessageCtx {
    orig_struct_ident,
    shadow_struct_ident: Some(shadow_struct_ident),
    non_ignored_fields,
    message_attrs,
  };

  let consistency_checks_impl = message_ctx.generate_consistency_checks();
  let validator_impl = message_ctx.generate_validator();
  let schema_impls = message_ctx.generate_schema_impls();

  let wrapped_items = wrap_with_imports(&[schema_impls, proto_conversion_impls, validator_impl]);

  Ok(quote! {
    #wrapped_items
    #consistency_checks_impl
  })
}

pub fn message_macro_direct(
  item: &mut ItemStruct,
  message_attrs: &MessageAttrs,
) -> Result<TokenStream2, Error> {
  let mut fields_data: Vec<FieldData> = Vec::new();
  let mut manually_set_tags: Vec<ParsedNum> = Vec::new();

  for field in item.fields.iter_mut() {
    let field_data_kind = process_field_data(FieldOrVariant::Field(field))?;

    if let FieldDataKind::Normal(data) = field_data_kind {
      if data.proto_field.is_enum() && !data.type_info.inner().is_int() {
        bail!(&data.type_info, "Enums must use `i32` in direct impls")
      }

      if data.proto_field.is_oneof() && !data.type_info.is_option() {
        bail!(
          &data.type_info,
          "Oneofs must be wrapped in `Option` in a direct impl"
        )
      }

      match data.type_info.type_.as_ref() {
        RustType::Box(inner) if data.proto_field.is_message() => {
          bail!(inner, "Boxed messages must be optional in a direct impl")
        }
        RustType::Other(inner) => {
          if data.proto_field.is_message() {
            bail!(
              &inner.path,
              "Messages must be wrapped in Option in direct impls"
            );
          }
        }
        _ => {}
      };

      if let Some(tag) = data.tag {
        manually_set_tags.push(tag);
      } else if let ProtoField::Oneof(OneofInfo { tags, .. }) = &data.proto_field {
        for tag in tags.iter().copied() {
          manually_set_tags.push(tag);
        }
      }

      fields_data.push(data);
    } else {
      bail!(
        field.require_ident()?,
        "Cannot use `ignore` in a direct impl. Use a proxied impl instead"
      );
    }
  }

  let used_ranges =
    build_unavailable_ranges(&message_attrs.reserved_numbers, &mut manually_set_tags)?;

  let mut tag_allocator = TagAllocator::new(&used_ranges);

  for (field, field_data) in item.fields.iter_mut().zip(fields_data.iter_mut()) {
    if !field_data.proto_field.is_oneof() && field_data.tag.is_none() {
      let new_tag = tag_allocator.next_tag(field_data.span)?;

      field_data.tag = Some(ParsedNum {
        num: new_tag,
        span: field_data.span,
      });
    };

    let prost_attr = field_data.as_prost_attr();
    field.attrs.push(prost_attr);
  }

  let message_ctx = MessageCtx {
    orig_struct_ident: &item.ident,
    shadow_struct_ident: None,
    non_ignored_fields: fields_data,
    message_attrs,
  };

  let consistency_checks_impl = message_ctx.generate_consistency_checks();
  let schema_impls = message_ctx.generate_schema_impls();
  let validator_impl = message_ctx.generate_validator();

  let wrapped_items = wrap_with_imports(&[schema_impls, validator_impl]);

  let output_tokens = quote! {
    #wrapped_items
    #consistency_checks_impl
  };

  Ok(output_tokens)
}
