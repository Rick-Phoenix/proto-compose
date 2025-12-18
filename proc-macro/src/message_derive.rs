use syn_utils::AsNamedField;

use crate::*;

pub fn process_message_derive(
  item: &mut ItemStruct,
  is_direct: bool,
) -> Result<TokenStream2, Error> {
  let message_attrs = process_derive_message_attrs(&item.ident, &item.attrs)?;

  match message_attrs.backend {
    Backend::Prost => process_message_derive_prost(item, message_attrs, is_direct),
    Backend::Protobuf => unimplemented!(),
  }
}

pub fn process_message_derive_prost(
  item: &mut ItemStruct,
  message_attrs: MessageAttrs,
  is_direct: bool,
) -> Result<TokenStream2, Error> {
  if is_direct {
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

  let mut validators_tokens = TokenStream2::new();
  let mut cel_rules_collection: Vec<TokenStream2> = Vec::new();
  let mut cel_checks_tokens: Vec<TokenStream2> = Vec::new();

  let mut proto_conversion_data = ProtoConversionImpl {
    source_ident: orig_struct_ident,
    target_ident: shadow_struct_ident,
    kind: InputItemKind::Struct,
    into_proto: ConversionData::new(&message_attrs.into_proto),
    from_proto: ConversionData::new(&message_attrs.from_proto),
  };

  let mut input_item = InputItem {
    impl_kind: ImplKind::Shadow {
      ignored_fields: &mut ignored_fields,
      proto_conversion_data: &mut proto_conversion_data,
    },
    validators_tokens: &mut validators_tokens,
    cel_rules_collection: &mut cel_rules_collection,
    cel_checks_tokens: &mut cel_checks_tokens,
  };

  for (src_field, dst_field) in orig_struct_fields.zip(shadow_struct_fields) {
    let src_field_ident = src_field.require_ident()?;
    let type_info = TypeInfo::from_type(&src_field.ty)?;
    let field_attrs = process_derive_field_attrs(src_field_ident, &type_info, &src_field.attrs)?;

    let field_data = ProcessFieldInput {
      field_or_variant: FieldOrVariant::Field(dst_field),
      input_item: &mut input_item,
      field_attrs,
    };

    let field_tokens = process_field(field_data)?;

    if !field_tokens.is_empty() {
      fields_tokens.push(field_tokens);
    }
  }

  let proto_conversion_impls = proto_conversion_data.generate_conversion_impls();

  // We strip away the ignored fields from the shadow struct
  if let Fields::Named(fields) = &mut shadow_struct.fields {
    let old_fields = std::mem::take(&mut fields.named);

    fields.named = old_fields
      .into_iter()
      .filter(|f| !ignored_fields.contains(f.ident.as_ref().unwrap()))
      .collect();
  }

  let (cel_check_impl, top_level_programs_ident) = if let Some(paths) = &message_attrs.cel_rules {
    let static_ident = format_ident!(
      "{}_CEL_RULES",
      ccase!(constant, orig_struct_ident.to_string())
    );

    let cel_check_impl =
      impl_cel_checks(shadow_struct_ident, &static_ident, paths, cel_checks_tokens);

    (Some(cel_check_impl), Some(static_ident))
  } else {
    (None, None)
  };

  let schema_impls = message_schema_impls(MessageSchemaImplsCtx {
    orig_struct_ident,
    shadow_struct_ident: Some(shadow_struct_ident),
    message_attrs: &message_attrs,
    entries_tokens: fields_tokens,
    top_level_programs_ident: top_level_programs_ident.as_ref(),
  });

  let shadow_struct_derives = message_attrs
    .shadow_derives
    .map(|list| quote! { #[#list] });

  let validator_impl = impl_validator(ValidatorImplCtx {
    target_ident: shadow_struct_ident,
    validators_tokens,
    top_level_programs_ident: top_level_programs_ident.as_ref(),
  });

  // prost::Message already implements Debug
  let output_tokens = quote! {
    #schema_impls

    #[derive(::prost::Message, Clone, PartialEq, ::protocheck_proc_macro::TryIntoCelValue)]
    #shadow_struct_derives
    #shadow_struct

    #proto_conversion_impls

    #validator_impl
    #cel_check_impl
  };

  Ok(wrap_with_imports(orig_struct_ident, output_tokens))
}

pub fn process_message_derive_direct(
  item: &mut ItemStruct,
  message_attrs: MessageAttrs,
) -> Result<TokenStream2, Error> {
  // prost::Message already implements Debug
  let prost_message_attr: Attribute = parse_quote!(#[derive(prost::Message, Clone, PartialEq, ::protocheck::macros::TryIntoCelValue)]);
  item.attrs.push(prost_message_attr);

  let mut fields_tokens: Vec<TokenStream2> = Vec::new();

  let mut validators_tokens = TokenStream2::new();
  let mut cel_rules_collection: Vec<TokenStream2> = Vec::new();
  let mut cel_checks_tokens: Vec<TokenStream2> = Vec::new();

  let mut input_item = InputItem {
    impl_kind: ImplKind::Direct,
    validators_tokens: &mut validators_tokens,
    cel_rules_collection: &mut cel_rules_collection,
    cel_checks_tokens: &mut cel_checks_tokens,
  };

  for src_field in item.fields.iter_mut() {
    let src_field_ident = src_field.require_ident()?;
    let type_info = TypeInfo::from_type(&src_field.ty)?;
    let field_attrs = process_derive_field_attrs(src_field_ident, &type_info, &src_field.attrs)?;

    if let FieldAttrData::Normal(data) = &field_attrs {
      match type_info.type_.as_ref() {
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
            bail!(inner, "Detected usage of `Option<Box<..>>`, but the field was not marked as a boxed message. Please use `#[proto(message(boxed))]` to mark it as a boxed message.");
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
    }

    let field_data = ProcessFieldInput {
      field_or_variant: FieldOrVariant::Field(src_field),
      input_item: &mut input_item,
      field_attrs,
    };

    let field_tokens = process_field(field_data)?;

    if !field_tokens.is_empty() {
      fields_tokens.push(field_tokens);
    }
  }

  let struct_ident = &item.ident;

  let (cel_check_impl, top_level_programs_ident) = if let Some(paths) = &message_attrs.cel_rules {
    let static_ident = format_ident!("{}_CEL_RULES", ccase!(constant, struct_ident.to_string()));

    let cel_check_impl = impl_cel_checks(struct_ident, &static_ident, paths, cel_checks_tokens);

    (Some(cel_check_impl), Some(static_ident))
  } else {
    (None, None)
  };

  let schema_impls = message_schema_impls(MessageSchemaImplsCtx {
    orig_struct_ident: struct_ident,
    shadow_struct_ident: None,
    message_attrs: &message_attrs,
    entries_tokens: fields_tokens,
    top_level_programs_ident: top_level_programs_ident.as_ref(),
  });

  let validator_impl = impl_validator(ValidatorImplCtx {
    target_ident: struct_ident,
    validators_tokens,
    top_level_programs_ident: top_level_programs_ident.as_ref(),
  });

  let output_tokens = quote! {
    #schema_impls
    #validator_impl
    #cel_check_impl
  };

  Ok(wrap_with_imports(struct_ident, output_tokens))
}
