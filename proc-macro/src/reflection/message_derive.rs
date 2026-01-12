use super::*;

#[derive(Default)]
pub struct ReflectionMsgData {
  pub fields_data: Vec<FieldDataKind>,
  pub top_level_rules: IterTokensOr<TokenStream2>,
  pub no_auto_test: SkipAutoTest,
  pub msg_name: String,
}

pub fn extract_fields_data_reflection(item: &mut ItemStruct) -> Result<ReflectionMsgData, Error> {
  let ItemStruct { fields, .. } = item;

  let mut msg_name: Option<String> = None;
  let mut no_auto_test = SkipAutoTest::No;

  for attr in &item.attrs {
    if attr.path().is_ident("proto") {
      attr.parse_nested_meta(|meta| {
        let ident = meta.ident_str()?;

        match ident.as_str() {
          "name" => {
            msg_name = Some(meta.parse_value::<LitStr>()?.value());
          }
          "no_auto_test" => {
            no_auto_test = true.into();
          }
          _ => drain_token_stream!(meta.input),
        };

        Ok(())
      })?
    }
  }

  let msg_name = msg_name.ok_or_else(|| error_call_site!("Missing message name"))?;

  let message_desc = match DESCRIPTOR_POOL.get_message_by_name(&msg_name) {
    Some(message) => message,
    None => {
      bail_call_site!("Message {msg_name} not found in the descriptor pool");
    }
  };

  let mut fields_data: Vec<FieldDataKind> = Vec::new();

  for field in fields {
    let field_span = field.ident.span();
    let ident = field.require_ident()?;
    let ident_str = ident.to_string();

    let mut proto_field: Option<ProtoField> = None;
    let mut found_enum_path: Option<Path> = None;

    for attr in &field.attrs {
      if attr.path().is_ident("prost") {
        attr.parse_nested_meta(|meta| {
          let ident_str = meta.ident_str()?;

          match ident_str.as_str() {
            "map" => {
              let val = meta.parse_value::<LitStr>()?.value();
              let (_, value) = val
                .split_once(", ")
                .ok_or_else(|| meta.error("Failed to parse map attribute"))?;

              if let Some((_, wrapped_path)) = value.split_once("enumeration") {
                let str_path = &wrapped_path[1..wrapped_path.len() - 1];
                found_enum_path = Some(syn::parse_str(str_path)?);
              }
            }
            "oneof" => {
              let path_str = meta.parse_value::<LitStr>()?;

              proto_field = Some(ProtoField::Oneof(OneofInfo {
                path: path_str.parse()?,
                tags: vec![],
                default: false,
                required: false,
              }));
            }
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

    let proto_name = rust_ident_to_proto_name(&ident_str);
    let type_info = TypeInfo::from_type(&field.ty)?;

    if let Some(ProtoField::Oneof(mut oneof)) = proto_field {
      let oneof_desc = message_desc
        .oneofs()
        .find(|oneof| oneof.name() == proto_name)
        .ok_or_else(|| error!(ident, "Oneof `{proto_name}` missing in the descriptor"))?;

      if let Some(oneof_rules) = get_oneof_rules(&oneof_desc) {
        oneof.required = oneof_rules.required();

        fields_data.push(FieldDataKind::Normal(FieldData {
          span: field_span,
          ident: ident.clone(),
          type_info,
          proto_name: proto_name.to_string(),
          ident_str,
          tag: None,
          validator: None,
          options: TokensOr::<TokenStream2>::vec(),
          proto_field: ProtoField::Oneof(oneof),
          from_proto: None,
          into_proto: None,
          deprecated: false,
        }));

        continue;
      }
    } else {
      let field_desc = message_desc
        .get_field_by_name(proto_name)
        .ok_or_else(|| error!(ident, "Field `{proto_name}` not found in the descriptor"))?;

      let proto_field = ProtoField::from_descriptor(&field_desc, &type_info, found_enum_path)?;

      let validator = if let Some(rules_ctx) = RulesCtx::from_descriptor(&field_desc, field_span) {
        let expr = match &proto_field {
          ProtoField::Map(proto_map) => rules_ctx.get_map_validator(proto_map),
          ProtoField::Oneof(_) => todo!(),
          ProtoField::Repeated(inner) => rules_ctx.get_repeated_validator(inner),
          ProtoField::Optional(inner) | ProtoField::Single(inner) => {
            rules_ctx.get_field_validator(inner)
          }
        };

        ValidatorTokens {
          expr: expr.into_built_validator(),
          is_fallback: false,
          span: field_span,
        }
      } else if let Some(fallback) = proto_field.default_validator_expr(field_span) {
        ValidatorTokens {
          expr: fallback,
          is_fallback: true,
          span: field_span,
        }
      } else {
        continue;
      };

      fields_data.push(FieldDataKind::Normal(FieldData {
        span: field_span,
        ident: ident.clone(),
        type_info,
        proto_name: proto_name.to_string(),
        ident_str,
        tag: Some(ParsedNum {
          num: field_desc.number().cast_signed(),
          span: Span::call_site(),
        }),
        validator: Some(validator),
        options: TokensOr::<TokenStream2>::vec(),
        proto_field,
        from_proto: None,
        into_proto: None,
        deprecated: false,
      }));
    }
  }

  let mut cel_rules = IterTokensOr::<TokenStream2>::vec();

  // Message Rules
  if let Some(message_rules) = get_message_rules(&message_desc) {
    for rule in message_rules.cel {
      let Rule {
        id,
        message,
        expression,
      } = rule;

      cel_rules
        .push(quote! { ::prelude::cel_program!(id = #id, msg = #message, expr = #expression) });
    }
  }

  Ok(ReflectionMsgData {
    fields_data,
    top_level_rules: cel_rules,
    no_auto_test,
    msg_name,
  })
}

fn get_message_rules(message_descriptor: &MessageDescriptor) -> Option<MessageRules> {
  if let ProstValue::Message(message_rules_msg) = message_descriptor
    .options()
    .get_extension(&MESSAGE_RULES_EXT_DESCRIPTOR)
    .as_ref()
  {
    Some(
      MessageRules::decode(message_rules_msg.encode_to_vec().as_slice())
        .expect("Could not decode message rules"),
    )
  } else {
    None
  }
}

fn get_oneof_rules(oneof_desc: &OneofDescriptor) -> Option<OneofRules> {
  if let ProstValue::Message(oneof_rules_msg) = oneof_desc
    .options()
    .get_extension(&ONEOF_RULES_EXT_DESCRIPTOR)
    .as_ref()
  {
    Some(
      OneofRules::decode(oneof_rules_msg.encode_to_vec().as_slice())
        .expect("Could not decode oneof rules"),
    )
  } else {
    None
  }
}

pub fn reflection_message_derive(item: &mut ItemStruct) -> TokenStream2 {
  let mut errors: Vec<Error> = Vec::new();

  let ReflectionMsgData {
    fields_data,
    top_level_rules: cel_rules,
    no_auto_test,
    msg_name,
  } = extract_fields_data_reflection(item).unwrap_or_default_and_push_error(&mut errors);

  let use_fallback = if errors.is_empty() {
    UseFallback::No
  } else {
    UseFallback::Yes
  };

  let validator_impl = wrap_with_imports(&[generate_message_validator(
    use_fallback,
    &item.ident,
    &fields_data,
    &cel_rules,
  )]);

  let consistency_checks = errors.is_empty().then(|| {
    generate_message_consistency_checks(
      &item.ident,
      &fields_data,
      no_auto_test,
      SkipOneofTagsCheck::Yes,
      &msg_name,
    )
  });

  let errors = errors.iter().map(|e| e.to_compile_error());

  quote! {
    #validator_impl
    #consistency_checks

    #(#errors)*
  }
}
