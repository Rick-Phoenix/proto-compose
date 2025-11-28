use crate::*;

pub enum OutputType {
  Keep,
  Change,
}

pub fn process_field(
  field: &mut Field,
  field_attrs: FieldAttrs,
  type_info: &TypeInfo,
  output_type: OutputType,
) -> Result<TokenStream2, Error> {
  let FieldAttrs {
    tag,
    validator,
    options,
    name,
    kind,
    oneof_tags,
    ..
  } = field_attrs;

  if kind.is_oneof() {
    let oneof_path = type_info.as_inner_option_path().ok_or(spanned_error!(
      &field.ty,
      "Oneofs must be wrapped in Option"
    ))?;

    let oneof_path_str = oneof_path.to_token_stream().to_string();
    let mut oneof_tags_str = String::new();

    for (i, tag) in oneof_tags.iter().enumerate() {
      oneof_tags_str.push_str(&tag.to_string());

      if i != oneof_tags.len() - 1 {
        oneof_tags_str.push_str(", ");
      }
    }

    let oneof_attr: Attribute =
      parse_quote!(#[prost(oneof = #oneof_path_str, tags = #oneof_tags_str)]);

    field.attrs.push(oneof_attr);

    return Ok(quote! {
      MessageEntry::Oneof(#oneof_path::to_oneof())
    });
  }

  let proto_type = match kind {
    ProtoFieldKind::Enum(path) => {
      // Handle the errors here and just say it can't be used for a map
      let enum_path = if let Some(path) = path {
        path
      } else {
        type_info
          .rust_type
          .inner_path()
          .ok_or(spanned_error!(
            &field.ty,
            "Failed to extract the inner type. Expected a type, or a type wrapped in Option or Vec"
          ))?
          .clone()
      };

      ProtoType::Enum(enum_path)
    }
    ProtoFieldKind::Message(path) => {
      let msg_path = if let MessagePath::Path(path) = path {
        path
      } else {
        let inner_type = type_info
          .rust_type
          .inner_path()
          .ok_or(spanned_error!(
            &field.ty,
            "Failed to extract the inner type. Expected a type, or a type wrapped in Option or Vec"
          ))?
          .clone();

        if path.is_suffixed() {
          append_proto_ident(inner_type)
        } else {
          inner_type
        }
      };

      ProtoType::Message(msg_path)
    }
    ProtoFieldKind::Map(proto_map) => {
      ProtoType::Map(set_map_proto_type(proto_map, &type_info.rust_type)?)
    }
    // No manually set type, let's try to infer it as a primitive
    // maybe use the larger error for any of these
    _ => match &type_info.rust_type {
      RustType::Option(path) => ProtoType::from_primitive(path)?,
      RustType::Boxed(path) => ProtoType::from_primitive(path)?,
      RustType::Vec(path) => ProtoType::from_primitive(path)?,
      RustType::Normal(path) => ProtoType::from_primitive(path)?,
      RustType::Map((k, v)) => {
        let keys = ProtoMapKeys::from_path(k)?;
        let values = ProtoMapValues::from_path(v).map_err(|_| spanned_error!(v, format!("Unrecognized proto map value type {}. If you meant to use an enum or a message, use the attribute", v.to_token_stream())))?;

        let proto_map = ProtoMap { keys, values };

        ProtoType::Map(set_map_proto_type(proto_map, &type_info.rust_type)?)
      }
    },
  };

  if let OutputType::Change = output_type {
    let proto_output_type_inner = proto_type.output_proto_type();

    // Get output type
    let proto_output_type_outer: Type = match &type_info.rust_type {
      RustType::Option(_) => parse_quote! { Option<#proto_output_type_inner> },
      RustType::Boxed(_) => parse_quote! { Option<Box<#proto_output_type_inner>> },
      RustType::Map(_) => parse_quote!( #proto_output_type_inner ),
      RustType::Vec(_) => parse_quote! { Vec<#proto_output_type_inner> },
      RustType::Normal(_) => parse_quote!( #proto_output_type_inner ),
    };

    field.ty = proto_output_type_outer;
  }

  let prost_attr = ProstAttrs::from_type_info(&type_info.rust_type, proto_type.clone(), tag);

  let field_prost_attr: Attribute = parse_quote!(#prost_attr);

  field.attrs.push(field_prost_attr);

  // Use new validator but with cardinality info
  let validator_tokens = if let Some(validator) = validator {
    type_info.validator_tokens(&validator, &proto_type)
  } else {
    quote! { None }
  };

  let field_type_tokens = type_info.as_proto_type_trait_expr(&proto_type);

  Ok(quote! {
    MessageEntry::Field(
      ProtoField {
        name: #name.to_string(),
        tag: #tag,
        options: #options,
        type_: #field_type_tokens,
        validator: #validator_tokens,
      }
    )
  })
}
