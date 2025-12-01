use crate::*;

pub enum OutputType {
  Keep,
  Change,
}

pub enum FieldOrVariant<'a> {
  Field(&'a mut Field),
  Variant(&'a mut Variant),
}

impl<'a> FieldOrVariant<'a> {
  pub fn inject_attr(&mut self, attr: Attribute) {
    match self {
      FieldOrVariant::Field(field) => field.attrs.push(attr),
      FieldOrVariant::Variant(variant) => variant.attrs.push(attr),
    }
  }

  pub fn change_type(&mut self, ty: Type) {
    let src_type = match self {
      FieldOrVariant::Field(field) => &mut field.ty,
      FieldOrVariant::Variant(variant) => {
        if let Fields::Unnamed(variant_fields) = &mut variant.fields {
          if variant_fields.unnamed.len() != 1 {
            panic!("Oneof variants must contain a single value");
          }

          &mut variant_fields.unnamed.first_mut().unwrap().ty
        } else {
          panic!("Oneof variants can only have one unnamed field")
        }
      }
    };

    *src_type = ty;
  }
}

pub fn process_field(
  field: &mut FieldOrVariant,
  field_attrs: FieldAttrs,
  type_info: &TypeInfo,
  output_type: OutputType,
) -> Result<TokenStream2, Error> {
  let FieldAttrs {
    tag,
    validator,
    options,
    name,
    ..
  } = field_attrs;

  if let ProtoType::Oneof {
    tags: oneof_tags,
    path: oneof_path,
    ..
  } = &type_info.proto_type
  {
    let oneof_path_str = oneof_path.to_token_stream().to_string();
    let mut oneof_tags_str = String::new();

    for (i, tag) in oneof_tags.iter().enumerate() {
      oneof_tags_str.push_str(&tag.to_string());

      if i != oneof_tags.len() - 1 {
        oneof_tags_str.push_str(", ");
      }
    }

    if oneof_tags.is_empty() {
      return Err(type_info.error("Oneof tags are empty"));
    }

    let oneof_attr: Attribute =
      parse_quote!(#[prost(oneof = #oneof_path_str, tags = #oneof_tags_str)]);

    field.inject_attr(oneof_attr);

    if let OutputType::Change = output_type {
      field.change_type(parse_quote! { Option<#oneof_path> });
    }

    // Early return
    return Ok(quote! {
      MessageEntry::Oneof(#oneof_path::to_oneof())
    });
  }

  let proto_type = &type_info.proto_type;

  if let OutputType::Change = output_type {
    let proto_output_type_inner = proto_type.output_proto_type();

    // Get output type
    let proto_output_type_outer: Type = match &type_info.rust_type {
      RustType::Option(_) => parse_quote! { Option<#proto_output_type_inner> },
      RustType::BoxedMsg(_) => parse_quote! { Option<Box<#proto_output_type_inner>> },
      RustType::Map(_) => parse_quote!( #proto_output_type_inner ),
      RustType::Vec(_) => parse_quote! { Vec<#proto_output_type_inner> },
      RustType::Normal(_) => parse_quote!( #proto_output_type_inner ),
      RustType::BoxedOneofVariant(path) => parse_quote! { Box<#proto_output_type_inner> },
    };

    field.change_type(proto_output_type_outer);
  }

  let prost_attr = ProstAttrs::from_type_info(&type_info, tag);

  let field_prost_attr: Attribute = parse_quote!(#prost_attr);

  field.inject_attr(field_prost_attr);

  // Use new validator but with cardinality info
  let validator_tokens = if let Some(validator) = validator {
    type_info.validator_tokens(&validator, &proto_type)
  } else {
    quote! { None }
  };

  let field_type_tokens = type_info.as_proto_type_trait_expr(&proto_type);

  let output = match field {
    FieldOrVariant::Field(_) => {
      quote! {
        MessageEntry::Field(
          ProtoField {
            name: #name.to_string(),
            tag: #tag,
            options: #options,
            type_: #field_type_tokens,
            validator: #validator_tokens,
          }
        )
      }
    }
    FieldOrVariant::Variant(_) => {
      quote! {
        ProtoField {
          name: #name.to_string(),
          tag: #tag,
          options: #options,
          type_: #field_type_tokens,
          validator: #validator_tokens,
        }
      }
    }
  };

  Ok(output)
}
