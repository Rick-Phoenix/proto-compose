use crate::*;

pub fn message_proc_macro(mut item: ItemStruct, macro_attrs: TokenStream2) -> TokenStream2 {
  let mut errors: Vec<Error> = Vec::new();

  let macro_args =
    MessageMacroArgs::parse(macro_attrs).unwrap_or_default_and_push_error(&mut errors);

  let message_attrs = process_message_attrs(&item.ident, macro_args, &item.attrs)
    .unwrap_or_default_and_push_error(&mut errors);

  let is_proxied = message_attrs.is_proxied;

  if is_proxied && !matches!(item.vis, Visibility::Public(_)) {
    item.vis = Visibility::Public(token::Pub::default());

    errors.push(error!(item.vis, "Proxy structs must be public"));
  }

  let mut proto_struct = is_proxied.then(|| create_shadow_struct(&item));

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

  let tag_allocator = TagAllocator::new(&used_ranges);

  let impl_kind = if is_proxied {
    ImplKind::Proxied
  } else {
    ImplKind::Direct
  };

  let struct_to_process = proto_struct.as_mut().unwrap_or(&mut item);

  ProcessFieldsData {
    impl_kind,
    fields: struct_to_process
      .fields
      .iter_mut()
      .map(|f| FieldOrVariant::Field(f)),
    fields_data: &mut fields_data,
    tag_allocator: Some(tag_allocator),
    container_attrs: ContainerAttrs::Message(&message_attrs),
    item_kind: ItemKind::Message,
  }
  .process_fields_data()
  .unwrap_or_default_and_push_error(&mut errors);

  let proto_derives = if !errors.is_empty() {
    FallbackImpls {
      orig_ident: &item.ident,
      proto_ident: proto_struct.as_ref().map(|ps| &ps.ident),
      kind: ItemKind::Message,
    }
    .fallback_derive_impls()
  } else if cfg!(feature = "cel") {
    // prost::Message already implements Debug and Default
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

  let main_struct_tokens = if let Some(proto_struct) = &mut proto_struct {
    if let Fields::Named(fields) = &mut proto_struct.fields {
      let old_fields = std::mem::take(&mut fields.named);

      fields.named = old_fields
        .into_iter()
        .zip(fields_data.iter())
        // Removing ignored fields
        .filter_map(|(field, data)| matches!(data, FieldDataKind::Normal(_)).then_some(field))
        .collect();
    }

    let extra_proto_derives = (!message_attrs.proto_derives.is_empty()).then(|| {
      let paths = &message_attrs.proto_derives;

      quote! { #[derive(#(#paths),*)] }
    });

    let conversions = ProtoConversions {
      proxy_ident: &item.ident,
      proto_ident: &proto_struct.ident,
      kind: ItemKind::Message,
      container_attrs: ContainerAttrs::Message(&message_attrs),
      fields: &fields_data,
    }
    .generate_proto_conversions();

    let forwarded_attrs = message_attrs.forwarded_attrs.iter().map(|meta| {
      quote_spanned! {meta.span()=>
        #[#meta]
      }
    });

    quote! {
      #[derive(::prelude::macros::Message)]
      #item

      #proto_derives
      #extra_proto_derives
      #(#forwarded_attrs)*
      #[allow(clippy::use_self)]
      #proto_struct

      #conversions
    }
  } else {
    quote! {
      #proto_derives
      #[derive(::prelude::macros::Message)]
      #item
    }
  };

  let message_ctx = MessageCtx {
    orig_struct_ident: &item.ident,
    shadow_struct_ident: proto_struct.as_ref().map(|ps| &ps.ident),
    fields_data,
    message_attrs: &message_attrs,
  };

  let consistency_checks = errors
    .is_empty()
    .then(|| message_ctx.generate_consistency_checks());
  let validator_impl = message_ctx.generate_validator();
  let schema_impls = message_ctx.generate_schema_impls();

  let wrapped_items = wrap_multiple_with_imports(&[schema_impls, validator_impl]);

  let errors = errors.iter().map(|e| e.to_compile_error());

  quote! {
    #main_struct_tokens
    #wrapped_items
    #consistency_checks
    #(#errors)*
  }
}

pub struct MessageCtx<'a> {
  pub orig_struct_ident: &'a Ident,
  pub shadow_struct_ident: Option<&'a Ident>,
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
  Proxied,
}

impl ImplKind {
  /// Returns `true` if the impl kind is [`Shadow`].
  ///
  /// [`Shadow`]: ImplKind::Shadow
  #[must_use]
  pub const fn is_shadow(self) -> bool {
    matches!(self, Self::Proxied)
  }
}

#[derive(Clone, Copy)]
pub enum ContainerAttrs<'a> {
  Message(&'a MessageAttrs),
  Oneof(&'a OneofAttrs),
}

impl ContainerAttrs<'_> {
  pub const fn custom_from_proto_expr(&self) -> Option<&PathOrClosure> {
    match self {
      ContainerAttrs::Message(message_attrs) => message_attrs.from_proto.as_ref(),
      ContainerAttrs::Oneof(oneof_attrs) => oneof_attrs.from_proto.as_ref(),
    }
  }

  pub const fn custom_into_proto_expr(&self) -> Option<&PathOrClosure> {
    match self {
      ContainerAttrs::Message(message_attrs) => message_attrs.into_proto.as_ref(),
      ContainerAttrs::Oneof(oneof_attrs) => oneof_attrs.into_proto.as_ref(),
    }
  }

  pub const fn has_custom_conversions(&self) -> bool {
    match self {
      ContainerAttrs::Message(message_attrs) => message_attrs.has_custom_conversions(),
      ContainerAttrs::Oneof(oneof_attrs) => {
        oneof_attrs.from_proto.is_some() && oneof_attrs.into_proto.is_some()
      }
    }
  }
}

pub struct ProcessFieldsData<'a, I>
where
  I: IntoIterator<Item = FieldOrVariant<'a>>,
{
  pub impl_kind: ImplKind,
  pub fields: I,
  pub fields_data: &'a mut [FieldDataKind],
  pub tag_allocator: Option<TagAllocator<'a>>,
  pub container_attrs: ContainerAttrs<'a>,
  pub item_kind: ItemKind,
}

impl<'a, I> ProcessFieldsData<'a, I>
where
  I: IntoIterator<Item = FieldOrVariant<'a>>,
{
  // Process:
  // - Allocate a tag if it's missing (and the item is not a oneof)
  // - Inject prost attribute
  // - (If proxied) Change output type
  // - Handle various kinds of wrong input
  pub fn process_fields_data(self) -> syn::Result<()> {
    let Self {
      impl_kind,
      fields,
      fields_data,
      mut tag_allocator,
      container_attrs,
      item_kind,
    } = self;

    for (mut dst_field, field_data) in fields.into_iter().zip(fields_data.iter_mut()) {
      // Skipping ignored fields
      let FieldDataKind::Normal(field_data) = field_data else {
        if impl_kind.is_shadow() {
          continue;
        } else {
          bail!(dst_field.ident()?, "Cannot ignore fields in a direct impl");
        }
      };

      for attr in &field_data.forwarded_attrs {
        dst_field.inject_attr(parse_quote!(#[#attr]));
      }

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
