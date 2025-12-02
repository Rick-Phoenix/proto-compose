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

  pub fn change_type(&mut self, ty: Type) -> Result<(), Error> {
    let src_type = match self {
      FieldOrVariant::Field(field) => &mut field.ty,
      FieldOrVariant::Variant(variant) => {
        if let Fields::Unnamed(variant_fields) = &mut variant.fields {
          if variant_fields.unnamed.len() != 1 {
            bail!(
              &variant.fields,
              "Oneof variants must contain a single unnamed value"
            );
          }

          &mut variant_fields.unnamed.first_mut().unwrap().ty
        } else {
          bail!(
            &variant.fields,
            "Oneof variants must contain a single unnamed value"
          );
        }
      }
    };

    *src_type = ty;

    Ok(())
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

  if let OutputType::Change = output_type {
    let proto_output_type = type_info.proto_field.output_proto_type();

    let proto_output_type_outer: Type = parse_quote! { #proto_output_type };

    field.change_type(proto_output_type_outer)?;
  }

  let prost_attr = type_info.as_prost_attr(tag);
  let field_prost_attr: Attribute = parse_quote!(#prost_attr);
  field.inject_attr(field_prost_attr);

  if let ProtoField::Oneof {
    path: oneof_path, ..
  } = &type_info.proto_field
  {
    // Early return
    return Ok(quote! {
      MessageEntry::Oneof(#oneof_path::to_oneof())
    });
  }

  let validator_tokens = if let Some(validator) = validator {
    type_info.validator_tokens(&validator)
  } else {
    quote! { None }
  };

  let field_type_tokens = type_info.proto_field.as_proto_type_trait_expr();

  let output = match field {
    FieldOrVariant::Field(_) => {
      quote! {
        MessageEntry::Field(
          ProtoField {
            name: #name.to_string(),
            tag: #tag,
            options: vec![ #(#options),* ],
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
          options: vec![ #(#options),* ],
          type_: #field_type_tokens,
          validator: #validator_tokens,
        }
      }
    }
  };

  Ok(output)
}
