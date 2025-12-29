use syn_utils::EnumVariant;

use crate::*;

pub fn process_oneof_derive(item: &mut ItemEnum, is_direct: bool) -> Result<TokenStream2, Error> {
  let oneof_attrs = process_oneof_attrs(&item.ident, &item.attrs)?;

  if is_direct {
    process_oneof_derive_direct(item, oneof_attrs)
  } else {
    process_oneof_derive_shadow(item, oneof_attrs)
  }
}

pub(crate) fn process_oneof_derive_shadow(
  item: &mut ItemEnum,
  oneof_attrs: OneofAttrs,
) -> Result<TokenStream2, Error> {
  let mut shadow_enum = create_shadow_enum(item);

  let orig_enum_ident = &item.ident;
  let shadow_enum_ident = &shadow_enum.ident;

  let mut output_tokens = TokenStream2::new();
  let mut variants_tokens: Vec<TokenStream2> = Vec::new();

  let orig_enum_variants = item.variants.iter_mut();
  let shadow_enum_variants = shadow_enum.variants.iter_mut();
  let mut ignored_variants: Vec<Ident> = Vec::new();

  let mut validators_tokens: Vec<TokenStream2> = Vec::new();
  let mut consistency_checks: Vec<TokenStream2> = Vec::new();

  let mut proto_conversion_data = ProtoConversionImpl {
    source_ident: orig_enum_ident,
    target_ident: shadow_enum_ident,
    kind: InputItemKind::Enum,
    into_proto: ConversionData::new(&oneof_attrs.into_proto),
    from_proto: ConversionData::new(&oneof_attrs.from_proto),
  };

  let mut manually_set_tags: Vec<ManuallySetTag> = Vec::new();
  let mut fields_attrs: Vec<FieldDataKind> = Vec::new();

  for src_variant in orig_enum_variants {
    let field_attrs = process_field_data(FieldOrVariant::Variant(src_variant))?;
    proto_conversion_data.handle_field_conversions(&field_attrs);

    match &field_attrs {
      FieldDataKind::Ignored { ident, .. } => ignored_variants.push(ident.clone()),
      FieldDataKind::Normal(data) => {
        if let Some(tag) = data.tag {
          manually_set_tags.push(ManuallySetTag {
            tag,
            field_span: src_variant.span(),
          });
        }
      }
    };

    fields_attrs.push(field_attrs);
  }

  sort_and_check_duplicate_tags(&mut manually_set_tags)?;

  for (dst_variant, field_attrs) in shadow_enum_variants.zip(fields_attrs) {
    let FieldDataKind::Normal(field_attrs) = field_attrs else {
      continue;
    };

    let field_tokens = field_attrs.generate_proto_impls(
      FieldOrVariant::Variant(dst_variant),
      &mut validators_tokens,
      &mut consistency_checks,
      None,
    )?;

    variants_tokens.push(field_tokens);
  }

  let proto_conversion_impls = proto_conversion_data.generate_conversion_impls();

  // We strip away the ignored variants from the shadow enum
  shadow_enum.variants = shadow_enum
    .variants
    .into_iter()
    .filter(|var| !ignored_variants.contains(&var.ident))
    .collect();

  let oneof_schema_impl = oneof_schema_impl(
    &oneof_attrs,
    orig_enum_ident,
    variants_tokens,
    &manually_set_tags,
  );

  let shadow_enum_derives = oneof_attrs
    .shadow_derives
    .map(|list| quote! { #[#list] });

  let consistency_checks_impl =
    impl_oneof_consistency_checks(shadow_enum_ident, consistency_checks);

  let validator_impl = impl_oneof_validator(OneofValidatorImplCtx {
    oneof_ident: shadow_enum_ident,
    validators_tokens,
  });

  let wrapped_items = wrap_with_imports(vec![
    oneof_schema_impl,
    proto_conversion_impls,
    validator_impl,
  ]);

  let derives = if cfg!(feature = "cel") {
    quote! { #[derive(::prelude::prost::Oneof, PartialEq, Clone, ::protocheck_proc_macro::TryIntoCelValue)] }
  } else {
    quote! { #[derive(::prelude::prost::Oneof, PartialEq, Clone)] }
  };

  // prost::Oneof already implements Debug
  output_tokens.extend(quote! {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #derives
    #shadow_enum_derives
    #shadow_enum

    #wrapped_items
    #consistency_checks_impl

    impl ::prelude::ProtoOneof for #shadow_enum_ident {
      fn name() -> &'static str {
        <#orig_enum_ident as ::prelude::ProtoOneof>::name()
      }

      fn tags() -> &'static [i32] {
        <#orig_enum_ident as ::prelude::ProtoOneof>::tags()
      }

      fn proto_schema() -> ::prelude::Oneof {
        <#orig_enum_ident as ::prelude::ProtoOneof>::proto_schema()
      }
    }
  });

  Ok(output_tokens)
}

pub(crate) fn process_oneof_derive_direct(
  item: &mut ItemEnum,
  oneof_attrs: OneofAttrs,
) -> Result<TokenStream2, Error> {
  let ItemEnum {
    attrs, variants, ..
  } = item;

  attrs.push(parse_quote!(#[allow(clippy::derive_partial_eq_without_eq)]));

  // prost::Oneof already implements Debug
  let prost_derive: Attribute = if cfg!(feature = "cel") {
    parse_quote!(#[derive(::prelude::prost::Oneof, PartialEq, Clone, ::protocheck_proc_macro::TryIntoCelValue)])
  } else {
    parse_quote!(#[derive(::prelude::prost::Oneof, PartialEq, Clone)])
  };

  attrs.push(prost_derive);

  let mut variants_tokens: Vec<TokenStream2> = Vec::new();

  let mut validators_tokens: Vec<TokenStream2> = Vec::new();
  let mut consistency_checks: Vec<TokenStream2> = Vec::new();

  let mut manually_set_tags: Vec<ManuallySetTag> = Vec::new();
  let mut fields_attrs: Vec<FieldData> = Vec::new();

  for variant in variants.iter_mut() {
    let field_attrs = process_field_data(FieldOrVariant::Variant(variant))?;
    let variant_type = variant.type_()?;

    if let FieldDataKind::Normal(data) = field_attrs {
      if let Some(tag) = data.tag {
        manually_set_tags.push(ManuallySetTag {
          tag,
          field_span: variant.span(),
        });
      }

      match data.type_info.type_.as_ref() {
        RustType::Box(_) => {
          if !matches!(
            data.proto_field,
            ProtoField::Single(ProtoType::Message { is_boxed: true, .. })
          ) {
            bail!(
              variant_type,
              "Box can only be used for messages in a native oneof"
            );
          }
        }

        // For unknown types such as messages
        RustType::Other(_) => {}

        _ => {
          if !data.type_info.type_.is_primitive() && !data.type_info.type_.is_bytes() {
            bail!(
              variant_type,
              "Unsupported Oneof variant type. If you want to use a custom type, you must use a proxied oneof with custom conversions"
            )
          }
        }
      };

      fields_attrs.push(data);
    } else {
      bail!(variant, "Cannot use `ignore` in direct impls");
    }
  }

  sort_and_check_duplicate_tags(&mut manually_set_tags)?;

  for (variant, field_attrs) in variants.iter_mut().zip(fields_attrs) {
    let field_tokens = field_attrs.generate_proto_impls(
      FieldOrVariant::Variant(variant),
      &mut validators_tokens,
      &mut consistency_checks,
      None,
    )?;

    variants_tokens.push(field_tokens);
  }

  let oneof_ident = &item.ident;

  let oneof_schema_impl = oneof_schema_impl(
    &oneof_attrs,
    oneof_ident,
    variants_tokens,
    &manually_set_tags,
  );

  let consistency_checks_impl = impl_oneof_consistency_checks(oneof_ident, consistency_checks);

  let validator_impl = impl_oneof_validator(OneofValidatorImplCtx {
    oneof_ident,
    validators_tokens,
  });

  let wrapped_items = wrap_with_imports(vec![oneof_schema_impl, validator_impl]);

  let output = quote! {
    #wrapped_items
    #consistency_checks_impl
  };

  Ok(output)
}
