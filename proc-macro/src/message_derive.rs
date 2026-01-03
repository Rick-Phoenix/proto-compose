use crate::*;

#[derive(Default)]
pub struct MessageMacroAttrs {
  pub is_proxied: bool,
  pub no_auto_test: bool,
  pub extern_path: Option<String>,
}

pub struct MessageCtx<'a, T: Borrow<FieldData>> {
  pub orig_struct_ident: &'a Ident,
  pub shadow_struct_ident: Option<&'a Ident>,
  pub non_ignored_fields: Vec<T>,
  pub message_attrs: MessageAttrs,
}

impl<'a, T: Borrow<FieldData>> MessageCtx<'a, T> {
  pub fn proto_struct_ident(&self) -> &'a Ident {
    self
      .shadow_struct_ident
      .unwrap_or(self.orig_struct_ident)
  }
}

pub fn process_message_derive(
  item: &mut ItemStruct,
  macro_attrs: MessageMacroAttrs,
) -> Result<TokenStream2, Error> {
  let message_attrs = process_derive_message_attrs(&item.ident, macro_attrs, &item.attrs)?;

  if message_attrs.is_proxied {
    process_message_derive_shadow(item, message_attrs)
  } else {
    process_message_derive_direct(item, message_attrs)
  }
}

pub fn process_message_derive_shadow(
  orig_struct: &mut ItemStruct,
  message_attrs: MessageAttrs,
) -> Result<TokenStream2, Error> {
  let mut shadow_struct = create_shadow_struct(orig_struct);

  let orig_struct_ident = &orig_struct.ident;
  let shadow_struct_ident = &shadow_struct.ident;

  let mut ignored_fields: Vec<Ident> = Vec::new();

  let mut proto_conversion_data = ProtoConversionImpl {
    source_ident: orig_struct_ident,
    target_ident: shadow_struct_ident,
    kind: InputItemKind::Message,
    into_proto: ConversionData::new(&message_attrs.into_proto),
    from_proto: ConversionData::new(&message_attrs.from_proto),
  };

  let mut fields_data: Vec<FieldDataKind> = Vec::new();
  let mut manually_set_tags: Vec<ManuallySetTag> = Vec::new();

  for field in orig_struct.fields.iter_mut() {
    let field_data_kind = process_field_data(FieldOrVariant::Field(field))?;

    proto_conversion_data.handle_field_conversions(&field_data_kind);

    match &field_data_kind {
      FieldDataKind::Ignored { .. } => {
        ignored_fields.push(field.require_ident()?.clone());
      }

      FieldDataKind::Normal(data) => {
        if let Some(tag) = data.tag {
          manually_set_tags.push(ManuallySetTag {
            tag,
            field_span: field.span(),
          });
        } else if let ProtoField::Oneof(OneofInfo { tags, .. }) = &data.proto_field {
          for tag in tags.iter().copied() {
            manually_set_tags.push(ManuallySetTag {
              tag,
              field_span: field.span(),
            });
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

    let tag = if field_data.proto_field.is_oneof() {
      // We don't need an actual tag for oneofs
      0
    } else {
      let new_tag = tag_allocator.next_tag_if_missing(field_data.tag, field_data.span)?;

      field_data.tag = Some(new_tag);
      new_tag
    };

    let prost_attr = field_data.proto_field.as_prost_attr(tag);
    dst_field.attrs.push(prost_attr);

    let prost_compatible_type = field_data.proto_field.output_proto_type();
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

  let wrapped_items = wrap_with_imports(vec![schema_impls, proto_conversion_impls, validator_impl]);

  // prost::Message already implements Debug and Default
  let derives = if cfg!(feature = "cel") {
    quote! { #[derive(::prelude::prost::Message, Clone, PartialEq, ::protocheck_proc_macro::TryIntoCelValue)] }
  } else {
    quote! { #[derive(::prelude::prost::Message, Clone, PartialEq)] }
  };

  let shadow_struct_derives = message_ctx
    .message_attrs
    .shadow_derives
    .map(|list| quote! { #[#list] });

  let output_tokens = quote! {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #derives
    #shadow_struct_derives
    #shadow_struct

    #wrapped_items
    #consistency_checks_impl
  };

  Ok(output_tokens)
}

pub fn process_message_derive_direct(
  item: &mut ItemStruct,
  message_attrs: MessageAttrs,
) -> Result<TokenStream2, Error> {
  item
    .attrs
    .push(parse_quote!(#[allow(clippy::derive_partial_eq_without_eq)]));

  // prost::Message already implements Debug and Default
  let prost_message_attr: Attribute = if cfg!(feature = "cel") {
    parse_quote!(#[derive(::prelude::prost::Message, Clone, PartialEq, ::protocheck::macros::TryIntoCelValue)])
  } else {
    parse_quote!(#[derive(::prelude::prost::Message, Clone, PartialEq)])
  };

  item.attrs.push(prost_message_attr);

  let mut fields_data: Vec<FieldData> = Vec::new();
  let mut manually_set_tags: Vec<ManuallySetTag> = Vec::new();

  for field in item.fields.iter_mut() {
    let field_data_kind = process_field_data(FieldOrVariant::Field(field))?;

    if let FieldDataKind::Normal(data) = field_data_kind {
      match data.type_info.type_.as_ref() {
        RustType::Box(inner) => {
          bail!(inner, "Boxed messages must be optional in a direct impl")
        }
        RustType::Option(inner) => {
          // This could be inferred, although it might be a bit too opaque
          if inner.is_box() && !data.proto_field.is_boxed_message() {
            bail!(
              inner,
              "Detected usage of `Option<Box<..>>`, but the field was not marked as a boxed message. Please use `#[proto(message(boxed))]` to mark it as a boxed message."
            );
          }
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
        manually_set_tags.push(ManuallySetTag {
          tag,
          field_span: field.span(),
        });
      } else if let ProtoField::Oneof(OneofInfo { tags, .. }) = &data.proto_field {
        for tag in tags.iter().copied() {
          manually_set_tags.push(ManuallySetTag {
            tag,
            field_span: field.span(),
          });
        }
      }

      fields_data.push(data);
    } else {
      bail!(
        field,
        "Cannot use `ignore` in a direct impl. Use a proxied impl instead"
      );
    }
  }

  let used_ranges =
    build_unavailable_ranges(&message_attrs.reserved_numbers, &mut manually_set_tags)?;

  let mut tag_allocator = TagAllocator::new(&used_ranges);

  for (field, field_data) in item.fields.iter_mut().zip(fields_data.iter_mut()) {
    let tag = if field_data.proto_field.is_oneof() {
      // We don't need an actual tag for oneofs
      0
    } else {
      let new_tag = tag_allocator.next_tag_if_missing(field_data.tag, field_data.span)?;

      field_data.tag = Some(new_tag);
      new_tag
    };

    // We change the type in direct impls as well,
    // mostly just to be able to use the real enum names
    // as opposed to just an opaque `i32`
    let prost_compatible_type = field_data.proto_field.output_proto_type();
    field.ty = prost_compatible_type;

    let prost_attr = field_data.proto_field.as_prost_attr(tag);
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

  let wrapped_items = wrap_with_imports(vec![schema_impls, validator_impl]);

  let output_tokens = quote! {
    #wrapped_items
    #consistency_checks_impl
  };

  Ok(output_tokens)
}
