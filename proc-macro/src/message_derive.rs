use crate::*;

pub(crate) fn process_message_derive_shadow(
  item: &mut ItemStruct,
  message_attrs: MessageAttrs,
) -> Result<TokenStream2, Error> {
  let mut shadow_struct = create_shadow_struct(item);

  let orig_struct_ident = &item.ident;
  let shadow_struct_ident = &shadow_struct.ident;

  let mut output_tokens = TokenStream2::new();
  let mut fields_tokens: Vec<TokenStream2> = Vec::new();

  let orig_struct_fields = item.fields.iter_mut();
  let shadow_struct_fields = shadow_struct.fields.iter_mut();
  let mut ignored_fields: Vec<Ident> = Vec::new();

  let mut from_proto_body = TokenStream2::new();
  let mut into_proto_body = TokenStream2::new();

  for (src_field, dst_field) in orig_struct_fields.zip(shadow_struct_fields) {
    let src_field_ident = src_field.ident.as_ref().expect("Expected named field");

    let rust_type = RustType::from_type(&src_field.ty, orig_struct_ident)?;

    let field_data = process_derive_field_attrs(src_field_ident, &rust_type, &src_field.attrs)?;

    let field_attrs = match field_data {
      FieldAttrData::Ignored { from_proto } => {
        ignored_fields.push(src_field.ident.clone().unwrap());

        if message_attrs.from_proto.is_none() {
          let from_proto_expr = field_from_proto_expression(FromProto {
            custom_expression: &from_proto,
            kind: FieldConversionKind::StructField {
              ident: src_field_ident,
            },
            type_info: None,
          })?;

          from_proto_body.extend(from_proto_expr);
        }

        continue;
      }
      FieldAttrData::Normal(field_attrs) => field_attrs,
    };

    let type_info = TypeInfo::from_type(rust_type, field_attrs.proto_field.clone(), &src_field.ty)?;

    if message_attrs.from_proto.is_none() {
      let from_proto_expr = field_from_proto_expression(FromProto {
        custom_expression: &field_attrs.into_proto,
        kind: FieldConversionKind::StructField {
          ident: src_field_ident,
        },
        type_info: Some(&type_info),
      })?;

      from_proto_body.extend(from_proto_expr);
    }

    let field_tokens = process_field(
      &mut FieldOrVariant::Field(dst_field),
      field_attrs.clone(),
      &type_info,
      OutputType::Change,
    )?;

    fields_tokens.push(field_tokens);

    if message_attrs.into_proto.is_none() {
      let field_into_proto = field_into_proto_expression(IntoProto {
        custom_expression: &field_attrs.into_proto,
        kind: FieldConversionKind::StructField {
          ident: src_field_ident,
        },
        type_info: &type_info,
      })?;

      into_proto_body.extend(field_into_proto);
    }
  }

  if let Fields::Named(fields) = &mut shadow_struct.fields {
    let old_fields = std::mem::take(&mut fields.named);

    fields.named = old_fields
      .into_iter()
      .filter(|f| !ignored_fields.contains(f.ident.as_ref().unwrap()))
      .collect();
  }

  let schema_impls = message_schema_impls(orig_struct_ident, &message_attrs, fields_tokens);

  let into_proto_impl = into_proto_impl(ItemConversion {
    source_ident: orig_struct_ident,
    target_ident: shadow_struct_ident,
    kind: ItemConversionKind::Struct,
    custom_expression: &message_attrs.into_proto,
    conversion_tokens: into_proto_body,
  });

  let from_proto_impl = from_proto_impl(ItemConversion {
    source_ident: orig_struct_ident,
    target_ident: shadow_struct_ident,
    kind: ItemConversionKind::Struct,
    custom_expression: &message_attrs.from_proto,
    conversion_tokens: from_proto_body,
  });

  output_tokens.extend(quote! {
    #schema_impls

    #[derive(prost::Message, Clone, PartialEq)]
    #shadow_struct

    #from_proto_impl
    #into_proto_impl

    impl AsProtoType for #shadow_struct_ident {
      fn proto_type() -> ProtoType {
        <#orig_struct_ident as AsProtoType>::proto_type()
      }
    }
  });

  Ok(output_tokens)
}

pub(crate) fn process_message_derive(item: &mut ItemStruct) -> Result<TokenStream2, Error> {
  let message_attrs = process_derive_message_attrs(&item.ident, &item.attrs)?;

  if message_attrs.direct {
    process_message_derive_direct(item, message_attrs)
  } else {
    process_message_derive_shadow(item, message_attrs)
  }
}

pub(crate) fn process_message_derive_direct(
  item: &mut ItemStruct,
  message_attrs: MessageAttrs,
) -> Result<TokenStream2, Error> {
  let prost_message_attr: Attribute = parse_quote!(#[derive(prost::Message, Clone, PartialEq)]);
  item.attrs.push(prost_message_attr);

  let mut output_tokens = TokenStream2::new();
  let mut fields_data: Vec<TokenStream2> = Vec::new();

  for src_field in item.fields.iter_mut() {
    let src_field_ident = src_field.ident.as_ref().expect("Expected named field");

    let rust_type = RustType::from_type(&src_field.ty, &item.ident)?;

    let field_data = process_derive_field_attrs(src_field_ident, &rust_type, &src_field.attrs)?;

    let field_attrs = match field_data {
      FieldAttrData::Ignored { .. } => {
        return Err(spanned_error!(
          src_field,
          "Fields cannot be ignored in a direct impl"
        ))
      }
      FieldAttrData::Normal(attrs) => attrs,
    };

    let type_info = TypeInfo::from_type(rust_type, field_attrs.proto_field.clone(), &src_field.ty)?;

    match &type_info.rust_type {
      RustType::Boxed(path) => {
        return Err(spanned_error!(
          path,
          "Boxed messages must be optional in a direct impl"
        ))
      }
      RustType::OptionBoxed(path) => {
        if !matches!(
          type_info.proto_field,
          ProtoField::Single(ProtoType::Message { is_boxed: true, .. })
        ) {
          return Err(spanned_error!(path, "Must be a boxed message"));
        }
      }
      RustType::Normal(path) => {
        if matches!(
          type_info.proto_field,
          ProtoField::Single(ProtoType::Message { .. })
        ) {
          return Err(spanned_error!(
            path,
            "Messages must be wrapped in Option in direct impls"
          ));
        }
      }
      _ => {}
    };

    let field_tokens = process_field(
      &mut FieldOrVariant::Field(src_field),
      field_attrs,
      &type_info,
      OutputType::Keep,
    )?;

    fields_data.push(field_tokens);
  }

  let schema_impls = message_schema_impls(&item.ident, &message_attrs, fields_data);

  output_tokens.extend(schema_impls);

  Ok(output_tokens)
}
