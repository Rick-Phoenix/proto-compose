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
  } = extract_fields_data(
    item.fields.len(),
    item
      .fields
      .iter_mut()
      .map(|f| FieldOrVariant::Field(f)),
  )
  .unwrap_or_default_and_push_error(&mut errors);

  let used_ranges =
    build_unavailable_ranges(&message_attrs.reserved_numbers, &mut manually_set_tags)
      .unwrap_or_default_and_push_error(&mut errors);

  let mut tag_allocator = TagAllocator::new(&used_ranges);

  let impl_kind = if is_proxied {
    ImplKind::Shadow
  } else {
    ImplKind::Direct
  };

  let struct_to_process = proxy_struct.as_mut().unwrap_or(&mut item);

  second_processing(
    impl_kind,
    struct_to_process
      .fields
      .iter_mut()
      .map(|f| FieldOrVariant::Field(f)),
    &mut fields_data,
    Some(&mut tag_allocator),
    ContainerAttrs::Message(&message_attrs),
    ItemKind::Message,
  )
  .unwrap_or_default_and_push_error(&mut errors);

  // prost::Message already implements Debug and Default
  let proto_derives = if !errors.is_empty() {
    FallbackImpls {
      orig_ident: &item.ident,
      shadow_ident: proxy_struct.as_ref().map(|ps| &ps.ident),
      kind: ItemKind::Message,
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
      kind: ItemKind::Message,
      into_proto: ConversionData::new(message_attrs.into_proto.as_ref()),
      from_proto: ConversionData::new(message_attrs.from_proto.as_ref()),
    }
    .generate_conversion_impls(&fields_data);

    output.extend(quote! {
      #[derive(::prelude::macros::Message)]
      #item

      #proto_derives
      #shadow_struct_derives
      #[allow(clippy::use_self)]
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

  let message_ctx = MessageCtx {
    orig_struct_ident: item.ident.clone(),
    shadow_struct_ident: proxy_struct.as_ref().map(|ps| ps.ident.clone()),
    fields_data,
    message_attrs: &message_attrs,
  };

  if errors.is_empty() {
    output.extend(message_ctx.generate_consistency_checks());
  }

  let validator_impl = message_ctx.generate_validator();
  let schema_impls = message_ctx.generate_schema_impls();

  let wrapped_items = wrap_with_imports(&[schema_impls, validator_impl]);

  output.extend(wrapped_items);

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
  /// Returns `true` if the impl kind is [`Shadow`].
  ///
  /// [`Shadow`]: ImplKind::Shadow
  #[must_use]
  pub const fn is_shadow(self) -> bool {
    matches!(self, Self::Shadow)
  }
}

pub enum ContainerAttrs<'a> {
  Message(&'a MessageAttrs),
  Oneof(&'a OneofAttrs),
}

impl ContainerAttrs<'_> {
  pub const fn has_custom_conversions(&self) -> bool {
    match self {
      ContainerAttrs::Message(message_attrs) => message_attrs.has_custom_conversions(),
      ContainerAttrs::Oneof(oneof_attrs) => {
        oneof_attrs.from_proto.is_some() && oneof_attrs.into_proto.is_some()
      }
    }
  }
}

#[allow(clippy::needless_pass_by_value)]
pub fn second_processing<'a, I>(
  impl_kind: ImplKind,
  fields: I,
  fields_data: &mut [FieldDataKind],
  mut tag_allocator: Option<&mut TagAllocator>,
  container_attrs: ContainerAttrs,
  item_kind: ItemKind,
) -> syn::Result<()>
where
  I: IntoIterator<Item = FieldOrVariant<'a>>,
{
  for (mut dst_field, field_data) in fields.into_iter().zip(fields_data.iter_mut()) {
    // Skipping ignored fields
    let FieldDataKind::Normal(field_data) = field_data else {
      if impl_kind.is_shadow() {
        continue;
      } else {
        bail!(dst_field.ident()?, "Cannot ignore fields in a direct impl");
      }
    };

    if !field_data.proto_field.is_oneof() && field_data.tag.is_none() {
      if let Some(tag_allocator) = tag_allocator.as_mut() {
        let new_tag = tag_allocator.next_tag(field_data.span)?;

        field_data.tag = Some(ParsedNum {
          num: new_tag,
          span: field_data.span,
        });
      } else {
        bail!(field_data.ident, "Field tag is missing");
      }
    };

    let prost_attr = field_data.as_prost_attr();
    dst_field.attributes_mut().push(prost_attr);

    if impl_kind.is_shadow() {
      let prost_compatible_type = field_data.output_proto_type(item_kind);
      *dst_field.type_mut()? = prost_compatible_type;

      if let ProtoField::Oneof(OneofInfo { default: false, .. }) = &field_data.proto_field
        && !field_data.type_info.is_option()
        && !field_data.has_custom_conversions()
        && !container_attrs.has_custom_conversions()
      {
        bail!(
          &field_data.type_info,
          "A oneof must be wrapped in `Option` unless a custom to/from proto implementation is provided or the `default` attribute is used"
        );
      }

      if item_kind.is_message()
        && let ProtoField::Single(ProtoType::Message(MessageInfo { default: false, .. })) =
          field_data.proto_field
        && !field_data.type_info.is_option()
        && !field_data.has_custom_conversions()
        && !container_attrs.has_custom_conversions()
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

      if item_kind.is_message() {
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
  }

  Ok(())
}

pub fn extract_fields_data<'a, I>(len: usize, fields: I) -> syn::Result<FieldsCtx>
where
  I: IntoIterator<Item = FieldOrVariant<'a>>,
{
  let mut fields_data: Vec<FieldDataKind> = Vec::with_capacity(len);
  let mut manually_set_tags: Vec<ParsedNum> = Vec::with_capacity(len);

  for field in fields {
    let field_data_kind = process_field_data(field)?;

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
