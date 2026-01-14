use super::*;

#[derive(Default)]
struct OneofDataReflection {
  pub no_auto_test: SkipAutoTest,
  pub fields_data: Vec<FieldDataKind>,
}

fn extract_oneof_data(item: &mut ItemEnum) -> Result<OneofDataReflection, Error> {
  let ItemEnum { variants, .. } = item;

  let mut parent_message: Option<String> = None;
  let mut no_auto_test = SkipAutoTest::No;

  for attr in &item.attrs {
    if attr.path().is_ident("proto") {
      attr.parse_nested_meta(|meta| {
        let ident_str = meta.ident_str()?;

        match ident_str.as_str() {
          "parent_message" => {
            parent_message = Some(meta.parse_value::<LitStr>()?.value());
          }
          "no_auto_test" => {
            no_auto_test = true.into();
          }
          _ => return Err(meta.error("Unknown attribute")),
        };

        Ok(())
      })?
    }
  }

  let parent_message =
    parent_message.ok_or_else(|| error_call_site!("Missing parent message name attribute"))?;

  let message_desc = match DESCRIPTOR_POOL.get_message_by_name(&parent_message) {
    Some(message) => message,
    None => {
      bail_call_site!("Message {parent_message} not found in the descriptor pool");
    }
  };

  let mut fields_data: Vec<FieldDataKind> = Vec::new();

  for variant in variants {
    let variant_span = variant.ident.span();

    let ident = &variant.ident;
    let ident_str = ident.to_string();

    let mut found_enum_path: Option<Path> = None;

    for attr in &variant.attrs {
      if attr.path().is_ident("prost") {
        attr.parse_nested_meta(|meta| {
          let ident_str = meta.ident_str()?;

          match ident_str.as_str() {
            "enumeration" => {
              let path_str = meta.parse_value::<LitStr>()?;

              found_enum_path = Some(path_str.parse()?);
            }
            _ => drain_token_stream!(meta.input),
          };

          Ok(())
        })?;
      }
    }

    let proto_name = to_snake_case(rust_ident_to_proto_name(&ident_str));

    let field_desc = message_desc
      .get_field_by_name(&proto_name)
      .ok_or_else(|| error!(ident, "Field `{proto_name}` not found in the descriptor"))?;

    let type_info = TypeInfo::from_type(variant.type_()?)?;
    let proto_field = ProtoField::from_descriptor(&field_desc, &type_info, found_enum_path)?;

    let validator = if let Some(rules_ctx) = RulesCtx::from_descriptor(&field_desc, variant_span) {
      let expr = match &proto_field {
        ProtoField::Optional(inner) | ProtoField::Single(inner) => {
          rules_ctx.get_field_validator(inner)
        }
        ProtoField::Map(_) => unreachable!("Maps cannot be used in oneofs"),
        ProtoField::Oneof(_) => unreachable!("Oneofs cannot be nested"),
        ProtoField::Repeated(_) => unreachable!("Repeated fields cannot be used in oneofs"),
      };

      ValidatorTokens {
        expr: expr.into_built_validator(),
        is_fallback: false,
        span: variant_span,
      }
    } else if let Some(fallback) = proto_field.default_validator_expr(variant_span) {
      fallback
    } else {
      continue;
    };

    fields_data.push(FieldDataKind::Normal(FieldData {
      span: variant_span,
      ident: ident.clone(),
      type_info,
      proto_name,
      ident_str,
      tag: Some(ParsedNum::with_default_span(
        field_desc.number().cast_signed(),
      )),
      validator: Some(validator),
      options: TokenStreamOr::vec(),
      proto_field,
      from_proto: None,
      into_proto: None,
      deprecated: false,
    }));
  }

  Ok(OneofDataReflection {
    no_auto_test,
    fields_data,
  })
}

pub fn reflection_oneof_derive(item: &mut ItemEnum) -> TokenStream2 {
  let mut errors: Vec<Error> = Vec::new();

  let OneofDataReflection {
    no_auto_test,
    fields_data,
  } = extract_oneof_data(item).unwrap_or_default_and_push_error(&mut errors);

  let use_fallback = if errors.is_empty() {
    UseFallback::No
  } else {
    UseFallback::Yes
  };

  let validator_impl = wrap_multiple_with_imports(&[generate_oneof_validator(
    use_fallback,
    &item.ident,
    &fields_data,
  )]);

  let consistency_checks = errors
    .is_empty()
    .then(|| generate_oneof_consistency_checks(&item.ident, &fields_data, no_auto_test));

  let errors = errors.iter().map(|e| e.to_compile_error());

  quote! {
    #validator_impl
    #consistency_checks

    #(#errors)*
  }
}
