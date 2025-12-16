use crate::*;

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

pub struct FieldCtx<'a> {
  pub field: &'a mut FieldOrVariant<'a>,
  pub field_attrs: &'a FieldAttrs,
  pub type_ctx: &'a TypeContext<'a>,
  pub field_ident: &'a Ident,
  pub validators_tokens: &'a mut TokenStream2,
  pub cel_rules: &'a mut Vec<TokenStream2>,
  pub cel_checks: &'a mut TokenStream2,
}

pub fn process_field(ctx: FieldCtx) -> Result<TokenStream2, Error> {
  let FieldCtx {
    field,
    field_attrs: FieldAttrs {
      tag,
      validator,
      options,
      name,
      ..
    },
    type_ctx,
    field_ident,
    validators_tokens,
    cel_rules,
    cel_checks,
  } = ctx;

  let proto_output_type = type_ctx.proto_field.output_proto_type();
  let proto_output_type_outer: Type = parse_quote! { #proto_output_type };

  field.change_type(proto_output_type_outer)?;

  let prost_attr = type_ctx.as_prost_attr(*tag);
  let field_prost_attr: Attribute = parse_quote!(#prost_attr);
  field.inject_attr(field_prost_attr);

  if let ProtoField::Oneof {
    path: oneof_path, ..
  } = &type_ctx.proto_field
  {
    // Early return
    return Ok(quote! {
      ::prelude::MessageEntry::Oneof(<#oneof_path as ::prelude::ProtoOneof>::proto_schema())
    });
  }

  let validator_schema_tokens = if let Some(validator) = validator {
    let field_type = type_ctx.proto_field.proto_kind_tokens();

    let field_context_tokens = quote! {
      ::prelude::FieldContext {
        name: #name,
        tag: #tag,
        field_type: #field_type,
        key_type: None,
        value_type: None,
        subscript: None,
        kind: Default::default(),
      }
    };

    let field_validator = type_ctx.validator_tokens(field_ident, field_context_tokens, validator);

    validators_tokens.extend(field_validator);

    let new_cel_rules = type_ctx.cel_rules_extractor(validator);

    cel_rules.push(new_cel_rules);

    let cel_check = type_ctx.cel_check_tokens(validator);
    cel_checks.extend(cel_check);

    let schema_expr = type_ctx.field_validator_schema(validator);

    quote! { Some(#schema_expr) }
  } else {
    quote! { None }
  };

  let field_type_tokens = type_ctx.proto_field.as_proto_type_trait_expr();
  let options_tokens = tokens_or_default!(options, quote! { vec![] });

  let output = match field {
    FieldOrVariant::Field(_) => {
      quote! {
        ::prelude::MessageEntry::Field(
          ::prelude::ProtoField {
            name: #name.to_string(),
            tag: #tag,
            options: #options_tokens,
            type_: #field_type_tokens,
            validator: #validator_schema_tokens,
          }
        )
      }
    }
    FieldOrVariant::Variant(_) => {
      quote! {
        ::prelude::ProtoField {
          name: #name.to_string(),
          tag: #tag,
          options: #options_tokens,
          type_: #field_type_tokens,
          validator: #validator_schema_tokens,
        }
      }
    }
  };

  Ok(output)
}
