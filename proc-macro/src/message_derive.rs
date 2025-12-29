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

  let mut fields_tokens: Vec<TokenStream2> = Vec::new();

  let orig_struct_fields = item.fields.iter_mut();
  let shadow_struct_fields = shadow_struct.fields.iter_mut();
  let mut ignored_fields: Vec<Ident> = Vec::new();

  let mut validators_tokens: Vec<TokenStream2> = Vec::new();
  let mut consistency_checks: Vec<TokenStream2> = Vec::new();

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
        } else if let ProtoField::Oneof { tags, path, .. } = &data.proto_field {
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

  for (dst_field, field_attrs) in shadow_struct_fields.zip(fields_with_ctx) {
    let FieldDataKind::Normal(field_attrs) = field_attrs else {
      continue;
    };

    let field_tokens = field_attrs.generate_proto_impls(
      FieldOrVariant::Field(dst_field),
      &mut validators_tokens,
      &mut consistency_checks,
      Some(&mut tag_allocator),
    )?;

    if !field_tokens.is_empty() {
      fields_tokens.push(field_tokens);
    }
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

  let consistency_checks_impl =
    if message_attrs.cel_rules.is_some() || !consistency_checks.is_empty() {
      Some(impl_message_consistency_checks(
        MessageConsistencyChecksCtx {
          item_ident: shadow_struct_ident,
          consistency_checks,
          no_auto_test: message_attrs.no_auto_test,
        },
      ))
    } else {
      None
    };

  let schema_impls = message_schema_impls(MessageSchemaImplsCtx {
    orig_struct_ident,
    shadow_struct_ident: Some(shadow_struct_ident),
    message_attrs: &message_attrs,
    entries_tokens: fields_tokens,
  });

  let shadow_struct_derives = message_attrs
    .shadow_derives
    .map(|list| quote! { #[#list] });

  let validator_impl = impl_message_validator(ValidatorImplCtx {
    target_ident: shadow_struct_ident,
    validators_tokens,
  });

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

  let mut fields_tokens: Vec<TokenStream2> = Vec::new();

  let mut validators_tokens: Vec<TokenStream2> = Vec::new();
  let mut consistency_checks: Vec<TokenStream2> = Vec::new();

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
      } else if let ProtoField::Oneof { tags, path, .. } = &data.proto_field {
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

  for (field, field_attrs) in item.fields.iter_mut().zip(fields_attrs) {
    let field_tokens = field_attrs.generate_proto_impls(
      FieldOrVariant::Field(field),
      &mut validators_tokens,
      &mut consistency_checks,
      Some(&mut tag_allocator),
    )?;

    fields_tokens.push(field_tokens);
  }

  let struct_ident = &item.ident;

  let consistency_checks_impl =
    if message_attrs.cel_rules.is_some() || !consistency_checks.is_empty() {
      Some(impl_message_consistency_checks(
        MessageConsistencyChecksCtx {
          item_ident: struct_ident,
          consistency_checks,
          no_auto_test: message_attrs.no_auto_test,
        },
      ))
    } else {
      None
    };

  let schema_impls = message_schema_impls(MessageSchemaImplsCtx {
    orig_struct_ident: struct_ident,
    shadow_struct_ident: None,
    message_attrs: &message_attrs,
    entries_tokens: fields_tokens,
  });

  let validator_impl = impl_message_validator(ValidatorImplCtx {
    target_ident: struct_ident,
    validators_tokens,
  });

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
