use crate::*;

pub struct ProcessFieldInput<'item, 'a, 'field> {
  pub field_or_variant: FieldOrVariant<'field>,
  pub input_item: &'a mut InputItem<'item, 'field>,
  pub field_attrs: FieldAttrData,
}

pub fn process_field(input: ProcessFieldInput) -> syn::Result<TokenStream2> {
  let ProcessFieldInput {
    field_or_variant,
    input_item:
      InputItem {
        impl_kind,
        validators_tokens,
        cel_checks_tokens,
        ..
      },
    field_attrs,
  } = input;

  let field_ident = field_or_variant.ident()?;
  let type_info = TypeInfo::from_type(field_or_variant.get_type()?)?;

  let field_attrs = match field_attrs {
    FieldAttrData::Ignored { from_proto } => {
      if let ImplKind::Shadow {
        ignored_fields,
        proto_conversion_data: proto_conversion_impls,
        ..
      } = impl_kind
      {
        ignored_fields.push(field_ident.clone());

        if !proto_conversion_impls
          .from_proto
          .has_custom_impl()
        {
          proto_conversion_impls.add_field_from_proto_impl(&from_proto, None, field_ident);
        }

        // If the field is ignored, we only need the (optional)
        // from_proto impl, we skip everything else
        return Ok(TokenStream2::new());
      } else {
        bail!(field_ident, "Cannot ignore fields in a direct impl")
      }
    }

    FieldAttrData::Normal(field_attrs) => *field_attrs,
  };

  if let ImplKind::Shadow {
    proto_conversion_data,
    ..
  } = impl_kind
  {
    if !proto_conversion_data.into_proto.has_custom_impl() {
      proto_conversion_data.add_field_into_proto_impl(
        &field_attrs.into_proto,
        &field_attrs.proto_field,
        field_ident,
      );
    }

    if !proto_conversion_data.from_proto.has_custom_impl() {
      proto_conversion_data.add_field_from_proto_impl(
        &field_attrs.from_proto,
        Some(&field_attrs.proto_field),
        field_ident,
      );
    }
  }

  let field_ctx = FieldCtx {
    field: field_or_variant,
    field_attrs,
    type_info,
    validators_tokens,
    cel_checks: cel_checks_tokens,
  };

  field_ctx.generate_proto_impls()
}
