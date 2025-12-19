use syn_utils::EnumVariant;

use crate::*;

pub fn process_oneof_derive(item: &mut ItemEnum, is_direct: bool) -> Result<TokenStream2, Error> {
  let oneof_attrs = process_oneof_attrs(&item.ident, &item.attrs)?;

  match oneof_attrs.backend {
    Backend::Prost => process_oneof_derive_prost(item, oneof_attrs, is_direct),
    Backend::Protobuf => unimplemented!(),
  }
}

pub fn process_oneof_derive_prost(
  item: &mut ItemEnum,
  oneof_attrs: OneofAttrs,
  is_direct: bool,
) -> Result<TokenStream2, Error> {
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
  let mut cel_checks_tokens: Vec<TokenStream2> = Vec::new();

  let mut proto_conversion_data = ProtoConversionImpl {
    source_ident: orig_enum_ident,
    target_ident: shadow_enum_ident,
    kind: InputItemKind::Enum,
    into_proto: ConversionData::new(&oneof_attrs.into_proto),
    from_proto: ConversionData::new(&oneof_attrs.from_proto),
  };

  let mut input_item = InputItem {
    impl_kind: ImplKind::Shadow {
      ignored_fields: &mut ignored_variants,
      proto_conversion_data: &mut proto_conversion_data,
    },
    validators_tokens: &mut validators_tokens,
    cel_checks_tokens: &mut cel_checks_tokens,
  };

  for (src_variant, dst_variant) in orig_enum_variants.zip(shadow_enum_variants) {
    let src_variant_ident = &src_variant.ident;
    let type_info = TypeInfo::from_type(src_variant.type_()?)?;
    let field_attrs =
      process_derive_field_attrs(src_variant_ident, &type_info, &src_variant.attrs)?;

    let field_tokens = process_field(ProcessFieldInput {
      field_or_variant: FieldOrVariant::Variant(dst_variant),
      input_item: &mut input_item,
      field_attrs,
    })?;

    if !field_tokens.is_empty() {
      variants_tokens.push(field_tokens);
    }
  }

  let proto_conversion_impls = proto_conversion_data.generate_conversion_impls();

  // We strip away the ignored variants from the shadow enum
  shadow_enum.variants = shadow_enum
    .variants
    .into_iter()
    .filter(|var| !ignored_variants.contains(&var.ident))
    .collect();

  let oneof_schema_impl = oneof_schema_impl(&oneof_attrs, orig_enum_ident, variants_tokens);

  let shadow_enum_derives = oneof_attrs
    .shadow_derives
    .map(|list| quote! { #[#list] });

  let cel_checks_impl = impl_oneof_cel_checks(shadow_enum_ident, cel_checks_tokens);

  let validator_impl = impl_oneof_validator(OneofValidatorImplCtx {
    oneof_ident: shadow_enum_ident,
    validators_tokens,
  });

  // prost::Oneof already implements Debug
  output_tokens.extend(quote! {
    #oneof_schema_impl

    #[derive(prost::Oneof, PartialEq, Clone, ::protocheck_proc_macro::TryIntoCelValue)]
    #shadow_enum_derives
    #shadow_enum

    #proto_conversion_impls
    #cel_checks_impl
    #validator_impl

    impl ::prelude::ProtoOneof for #shadow_enum_ident {
      fn proto_schema() -> ::prelude::Oneof {
        <#orig_enum_ident as ::prelude::ProtoOneof>::proto_schema()
      }
    }
  });

  Ok(wrap_with_imports(orig_enum_ident, output_tokens))
}

pub(crate) fn process_oneof_derive_direct(
  item: &mut ItemEnum,
  oneof_attrs: OneofAttrs,
) -> Result<TokenStream2, Error> {
  let ItemEnum {
    attrs, variants, ..
  } = item;

  // prost::Oneof already implements Debug
  let prost_derive: Attribute = parse_quote!(#[derive(prost::Oneof, PartialEq, Clone)]);
  attrs.push(prost_derive);

  let mut variants_tokens: Vec<TokenStream2> = Vec::new();

  let mut validators_tokens: Vec<TokenStream2> = Vec::new();
  let mut cel_checks_tokens: Vec<TokenStream2> = Vec::new();

  let mut input_item = InputItem {
    impl_kind: ImplKind::Direct,
    validators_tokens: &mut validators_tokens,
    cel_checks_tokens: &mut cel_checks_tokens,
  };

  for variant in variants {
    let variant_ident = &variant.ident;
    let variant_type = variant.type_()?;
    let type_info = TypeInfo::from_type(variant_type)?;
    let field_attrs = process_derive_field_attrs(variant_ident, &type_info, &variant.attrs)?;

    if let FieldAttrData::Normal(data) = &field_attrs {
      match type_info.type_.as_ref() {
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
          if !type_info.type_.is_primitive() && !type_info.type_.is_bytes() {
            bail!(variant_type, "Unsupported Oneof variant type. If you want to use a custom type, you must use a proxied oneof with custom conversions")
          }
        }
      };
    }

    let field_tokens = process_field(ProcessFieldInput {
      field_or_variant: FieldOrVariant::Variant(variant),
      input_item: &mut input_item,
      field_attrs,
    })?;

    if !field_tokens.is_empty() {
      variants_tokens.push(field_tokens);
    }
  }

  let oneof_ident = &item.ident;

  let oneof_schema_impl = oneof_schema_impl(&oneof_attrs, oneof_ident, variants_tokens);

  let cel_checks_impl = impl_oneof_cel_checks(oneof_ident, cel_checks_tokens);

  let validator_impl = impl_oneof_validator(OneofValidatorImplCtx {
    oneof_ident,
    validators_tokens,
  });

  let output = quote! {
    #oneof_schema_impl
    #cel_checks_impl
    #validator_impl
  };

  Ok(wrap_with_imports(&item.ident, output))
}
