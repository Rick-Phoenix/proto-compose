use crate::*;

pub fn process_message_derive(item: &mut ItemStruct) -> Result<TokenStream2, Error> {
  let message_attrs = process_derive_message_attrs(&item.ident, &item.attrs)?;

  match message_attrs.backend {
    Backend::Prost => process_message_derive_prost(item, message_attrs),
    Backend::Protobuf => unimplemented!(),
  }
}

pub fn process_message_derive_prost(
  item: &mut ItemStruct,
  message_attrs: MessageAttrs,
) -> Result<TokenStream2, Error> {
  if message_attrs.direct {
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

  let mut output_tokens = TokenStream2::new();
  let mut fields_tokens: Vec<TokenStream2> = Vec::new();

  let orig_struct_fields = item.fields.iter_mut();
  let shadow_struct_fields = shadow_struct.fields.iter_mut();
  let mut ignored_fields: Vec<Ident> = Vec::new();

  let mut from_proto_body = TokenStream2::new();
  let mut into_proto_body = TokenStream2::new();

  let mut validator_tokens = TokenStream2::new();
  let mut cel_rules_collection: Vec<TokenStream2> = Vec::new();

  for (src_field, dst_field) in orig_struct_fields.zip(shadow_struct_fields) {
    let src_field_ident = src_field
      .ident
      .as_ref()
      .ok_or(spanned_error!(&src_field, "Expected a named field"))?;

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
      FieldAttrData::Normal(field_attrs) => *field_attrs,
    };

    let type_info = TypeInfo::from_type(rust_type, field_attrs.proto_field.clone())?;

    if message_attrs.from_proto.is_none() {
      let from_proto_expr = field_from_proto_expression(FromProto {
        custom_expression: &field_attrs.from_proto,
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
    )?;

    fields_tokens.push(field_tokens);

    if let Some(validator) = &field_attrs.validator {
      let field_tag = field_attrs.tag;
      let field_name = &field_attrs.name;
      let field_type = type_info.proto_field.proto_kind_tokens();

      let field_context_tokens = quote! {
        ::prelude::FieldContext {
          name: #field_name,
          tag: #field_tag,
          field_type: #field_type,
          key_type: None,
          value_type: None,
          subscript: None,
          kind: Default::default(),
        }
      };

      let field_validator =
        type_info.validator_tokens(src_field_ident, field_context_tokens, validator);

      validator_tokens.extend(field_validator);

      let cel_rules = type_info.cel_rules_extractor(validator);

      cel_rules_collection.push(cel_rules);
    }

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

  let schema_impls = message_schema_impls(
    orig_struct_ident,
    &message_attrs,
    fields_tokens,
    cel_rules_collection,
  );

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

  let shadow_struct_derives = message_attrs
    .shadow_derives
    .map(|list| quote! { #[#list] });

  let cel_rules = &message_attrs
    .validator
    .map_or_else(|| quote! { vec![] }, |v| v.to_token_stream());

  output_tokens.extend(quote! {
    #schema_impls

    #[derive(::prost::Message, Clone, PartialEq, ::protocheck_proc_macro::TryIntoCelValue)]
    #shadow_struct_derives
    #shadow_struct

    #from_proto_impl
    #into_proto_impl

    impl #shadow_struct_ident {
      #[doc(hidden)]
      fn __validate_internal(&self, field_context: Option<&FieldContext>, parent_elements: &mut Vec<FieldPathElement>) -> Result<(), Vec<::proto_types::protovalidate::Violation>> {
        use ::prelude::{ProtoValidator, Validator, ValidationResult, field_context::Violations};

        let mut violations = Vec::new();

        if let Some(field_context) = field_context {
          parent_elements.push(FieldPathElement {
            field_number: Some(field_context.tag),
            field_name: Some(field_context.name.to_string()),
            field_type: Some(Type::Message as i32),
            key_type: field_context.key_type.map(|t| t as i32),
            value_type: field_context.value_type.map(|t| t as i32),
            subscript: field_context.subscript.clone(),
          });
        }

        let mut cel_rules: Vec<CelRule> = #cel_rules;

        for rule in cel_rules {
          let program = CelProgram::new(rule);

          match program.execute(self.clone()) {
            Ok(was_successful) => {
              if !was_successful {
                violations.add_cel(&program.rule, None, parent_elements);
              }
            }
            Err(e) => violations.push(e.into_violation(&program.rule, None, parent_elements))
          };
        }


        #validator_tokens

        if field_context.is_some() {
          parent_elements.pop();
        }

        if violations.is_empty() {
          Ok(())
        } else {
          Err(violations)
        }
      }

      pub fn validate(&self) -> Result<(), Vec<::proto_types::protovalidate::Violation>> {
        self.__validate_internal(None, &mut vec![])
      }

      pub fn nested_validate(&self, field_context: &FieldContext, parent_elements: &mut Vec<FieldPathElement>) -> Result<(), Vec<::proto_types::protovalidate::Violation>> {
        self.__validate_internal(Some(field_context), parent_elements)
      }
    }

    impl ::prelude::ProtoValidator<#shadow_struct_ident> for #shadow_struct_ident {
      type Target = Self;
      type Validator = ::prelude::MessageValidator<Self>;
      type Builder = ::prelude::MessageValidatorBuilder<Self>;

      fn builder() -> Self::Builder {
        ::prelude::MessageValidator::builder()
      }
    }

    impl ::prelude::ProtoMessage for #shadow_struct_ident {
      fn cel_rules() -> Vec<Arc<[CelRule]>> {
        #orig_struct_ident::cel_rules()
      }
      fn proto_path() -> ::prelude::ProtoPath {
        <#orig_struct_ident as ::prelude::ProtoMessage>::proto_path()
      }

      fn proto_schema() -> ::prelude::Message {
        #orig_struct_ident::proto_schema()
      }

      fn validate(&self) -> Result<(), Vec<::proto_types::protovalidate::Violation>> {
        self.validate()
      }

      fn nested_validate(&self, field_context: &FieldContext, parent_elements: &mut Vec<FieldPathElement>) -> Result<(), Vec<::proto_types::protovalidate::Violation>> {
        self.nested_validate(field_context, parent_elements)
      }
    }

    impl ::prelude::AsProtoType for #shadow_struct_ident {
      fn proto_type() -> ::prelude::ProtoType {
        <#orig_struct_ident as ::prelude::AsProtoType>::proto_type()
      }
    }
  });

  Ok(output_tokens)
}

pub fn process_message_derive_direct(
  item: &mut ItemStruct,
  message_attrs: MessageAttrs,
) -> Result<TokenStream2, Error> {
  let prost_message_attr: Attribute = parse_quote!(#[derive(prost::Message, Clone, PartialEq)]);
  item.attrs.push(prost_message_attr);

  let mut output_tokens = TokenStream2::new();
  let mut fields_data: Vec<TokenStream2> = Vec::new();

  for src_field in item.fields.iter_mut() {
    let src_field_ident = src_field
      .ident
      .as_ref()
      .ok_or(spanned_error!(&src_field, "Expected a named field"))?;

    let rust_type = RustType::from_type(&src_field.ty, &item.ident)?;

    let field_data = process_derive_field_attrs(src_field_ident, &rust_type, &src_field.attrs)?;

    let field_attrs = match field_data {
      FieldAttrData::Ignored { .. } => {
        return Err(spanned_error!(
          src_field,
          "Fields cannot be ignored in a direct impl"
        ))
      }
      FieldAttrData::Normal(attrs) => *attrs,
    };

    let type_info = TypeInfo::from_type(rust_type, field_attrs.proto_field.clone())?;

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
    )?;

    fields_data.push(field_tokens);
  }

  let schema_impls = message_schema_impls(&item.ident, &message_attrs, fields_data, Vec::new());

  output_tokens.extend(schema_impls);

  let struct_ident = &item.ident;

  output_tokens.extend(quote! {
    impl ::prelude::ProtoValidator<#struct_ident> for #struct_ident {
      type Target = Self;
      type Validator = ::prelude::MessageValidator<Self>;
      type Builder = ::prelude::MessageValidatorBuilder<Self>;

      fn builder() -> Self::Builder {
        ::prelude::MessageValidator::builder()
      }
    }
  });

  Ok(output_tokens)
}
