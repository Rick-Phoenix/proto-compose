use syn_utils::AsNamedField;

use crate::*;

#[derive(Default)]
pub struct MessageMacroAttrs {
  pub is_direct: bool,
  pub no_auto_test: bool,
  pub extern_path: Option<String>,
}

pub fn process_message_derive(
  item: &mut ItemStruct,
  macro_attrs: MessageMacroAttrs,
) -> Result<TokenStream2, Error> {
  let message_attrs = process_derive_message_attrs(&item.ident, macro_attrs, &item.attrs)?;

  if message_attrs.is_direct {
    process_message_derive_direct(item, message_attrs)
  } else {
    process_message_derive_shadow(item, message_attrs)
  }
}

pub fn process_message_derive_shadow(
  item: &mut ItemStruct,
  message_attrs: MessageAttrs,
) -> Result<TokenStream2, Error> {
  let mut shadow_struct = create_shadow_struct(item);

  let orig_struct_ident = &item.ident;
  let shadow_struct_ident = &shadow_struct.ident;

  let orig_struct_fields = item.fields.iter_mut();
  let shadow_struct_fields = shadow_struct.fields.iter_mut();
  let mut ignored_fields: Vec<Ident> = Vec::new();

  let mut proto_conversion_data = ProtoConversionImpl {
    source_ident: orig_struct_ident,
    target_ident: shadow_struct_ident,
    kind: InputItemKind::Struct,
    into_proto: ConversionData::new(&message_attrs.into_proto),
    from_proto: ConversionData::new(&message_attrs.from_proto),
  };

  let mut fields_with_ctx: Vec<FieldDataKind> = Vec::new();
  let mut manually_set_tags: Vec<ManuallySetTag> = Vec::new();
  let mut oneofs: Vec<OneofCheckCtx> = Vec::new();

  for field in orig_struct_fields {
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
        } else if let ProtoField::Oneof(OneofInfo { tags, path, .. }) = &data.proto_field {
          for tag in tags.iter().copied() {
            manually_set_tags.push(ManuallySetTag {
              tag,
              field_span: field.span(),
            });
          }

          oneofs.push(OneofCheckCtx {
            path: path.to_token_stream(),
            tags: tags.clone(),
          });
        }
      }
    };

    fields_with_ctx.push(field_data_kind);
  }

  let used_ranges =
    build_unavailable_ranges2(&message_attrs.reserved_numbers, &mut manually_set_tags)?;

  let mut tag_allocator = TagAllocator::new(&used_ranges);

  for (dst_field, field_attrs) in shadow_struct_fields.zip(fields_with_ctx.iter_mut()) {
    let FieldDataKind::Normal(field_attrs) = field_attrs else {
      continue;
    };

    let prost_compatible_type = field_attrs.proto_field.output_proto_type();
    dst_field.ty = prost_compatible_type;

    let tag = match field_attrs.tag {
      Some(tag) => tag,
      None => {
        let new_tag = tag_allocator
          .next_tag()
          .map_err(|e| error_with_span!(field_attrs.span, "{e}"))?;

        field_attrs.tag = Some(new_tag);
        new_tag
      }
    };

    let prost_attr = field_attrs.proto_field.as_prost_attr(tag);
    dst_field.attrs.push(prost_attr);
  }

  let proto_conversion_impls = proto_conversion_data.generate_conversion_impls();
  let validated_conversion_impls = proto_conversion_data.create_validated_conversion_helpers();

  // We strip away the ignored fields from the shadow struct
  if let Fields::Named(fields) = &mut shadow_struct.fields {
    let old_fields = std::mem::take(&mut fields.named);

    fields.named = old_fields
      .into_iter()
      .filter(|f| !ignored_fields.contains(f.ident.as_ref().unwrap()))
      .collect();
  }

  let non_ignored_fields: Vec<&FieldData> = fields_with_ctx
    .iter()
    .filter_map(|f| f.as_normal())
    .collect();

  let consistency_checks_impl = impl_message_consistency_checks(
    shadow_struct_ident,
    &non_ignored_fields,
    message_attrs.no_auto_test,
  );

  let validator_impl = impl_message_validator(shadow_struct_ident, &non_ignored_fields);

  let schema_impls = message_schema_impls(
    orig_struct_ident,
    Some(shadow_struct_ident),
    &message_attrs,
    &non_ignored_fields,
  );

  let shadow_struct_derives = message_attrs
    .shadow_derives
    .map(|list| quote! { #[#list] });

  let wrapped_items = wrap_with_imports(vec![
    schema_impls,
    proto_conversion_impls,
    validated_conversion_impls,
    validator_impl,
  ]);

  let oneof_tags_check =
    generate_oneof_tags_check(shadow_struct_ident, message_attrs.no_auto_test, oneofs);

  let derives = if cfg!(feature = "cel") {
    quote! { #[derive(::prelude::prost::Message, Clone, PartialEq, ::protocheck_proc_macro::TryIntoCelValue)] }
  } else {
    quote! { #[derive(::prelude::prost::Message, Clone, PartialEq)] }
  };

  // prost::Message already implements Debug
  let output_tokens = quote! {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #derives
    #shadow_struct_derives
    #shadow_struct

    #wrapped_items
    #oneof_tags_check
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

  // prost::Message already implements Debug
  let prost_message_attr: Attribute = if cfg!(feature = "cel") {
    parse_quote!(#[derive(::prelude::prost::Message, Clone, PartialEq, ::protocheck::macros::TryIntoCelValue)])
  } else {
    parse_quote!(#[derive(::prelude::prost::Message, Clone, PartialEq)])
  };

  item.attrs.push(prost_message_attr);

  let mut fields_attrs: Vec<FieldData> = Vec::new();
  let mut manually_set_tags: Vec<ManuallySetTag> = Vec::new();
  let mut oneofs: Vec<OneofCheckCtx> = Vec::new();

  for field in item.fields.iter_mut() {
    let field_attrs = process_field_data(FieldOrVariant::Field(field))?;

    if let FieldDataKind::Normal(data) = field_attrs {
      match data.type_info.type_.as_ref() {
        RustType::Box(inner) => {
          bail!(inner, "Boxed messages must be optional in a direct impl")
        }
        RustType::Option(inner) => {
          if inner.is_box()
            && !matches!(
              data.proto_field,
              ProtoField::Single(ProtoType::Message { is_boxed: true, .. })
            )
          {
            bail!(
              inner,
              "Detected usage of `Option<Box<..>>`, but the field was not marked as a boxed message. Please use `#[proto(message(boxed))]` to mark it as a boxed message."
            );
          }
        }
        RustType::Other(inner) => {
          if matches!(
            data.proto_field,
            ProtoField::Single(ProtoType::Message { .. })
          ) {
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
      } else if let ProtoField::Oneof(OneofInfo { tags, path, .. }) = &data.proto_field {
        for tag in tags.iter().copied() {
          manually_set_tags.push(ManuallySetTag {
            tag,
            field_span: field.span(),
          });
        }

        oneofs.push(OneofCheckCtx {
          path: path.to_token_stream(),
          tags: tags.clone(),
        });
      }

      fields_attrs.push(data);
    } else {
      bail!(field, "Cannot use `ignore` in a direct impl");
    }
  }

  let used_ranges =
    build_unavailable_ranges2(&message_attrs.reserved_numbers, &mut manually_set_tags)?;

  let mut tag_allocator = TagAllocator::new(&used_ranges);

  for (field, field_attrs) in item
    .fields
    .iter_mut()
    .zip(fields_attrs.iter_mut())
  {
    let tag = match field_attrs.tag {
      Some(tag) => tag,
      None => {
        let new_tag = tag_allocator
          .next_tag()
          .map_err(|e| error_with_span!(field_attrs.span, "{e}"))?;

        field_attrs.tag = Some(new_tag);
        new_tag
      }
    };

    let prost_compatible_type = field_attrs.proto_field.output_proto_type();
    field.ty = prost_compatible_type;

    let prost_attr = field_attrs.proto_field.as_prost_attr(tag);
    field.attrs.push(prost_attr);
  }

  let struct_ident = &item.ident;

  let consistency_checks_impl =
    impl_message_consistency_checks(struct_ident, &fields_attrs, message_attrs.no_auto_test);

  let schema_impls = message_schema_impls(struct_ident, None, &message_attrs, &fields_attrs);

  let validator_impl = impl_message_validator(struct_ident, &fields_attrs);

  let oneof_tags_check =
    generate_oneof_tags_check(struct_ident, message_attrs.no_auto_test, oneofs);

  let wrapped_items = wrap_with_imports(vec![schema_impls, validator_impl]);

  let output_tokens = quote! {
    #wrapped_items
    #oneof_tags_check
    #consistency_checks_impl
  };

  Ok(output_tokens)
}
