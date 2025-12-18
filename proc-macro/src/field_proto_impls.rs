use crate::*;

pub struct FieldCtx<'a, 'field> {
  pub field: &'a mut FieldOrVariant<'field>,
  pub field_attrs: &'a FieldAttrs,
  pub type_ctx: &'a TypeContext<'a>,
  pub validators_tokens: &'a mut TokenStream2,
  pub cel_rules: &'a mut Vec<TokenStream2>,
  pub cel_checks: &'a mut Vec<TokenStream2>,
}

impl<'a, 'field> FieldCtx<'a, 'field> {
  pub fn generate_proto_impls(self) -> syn::Result<TokenStream2> {
    let FieldCtx {
      field,
      field_attrs:
        FieldAttrs {
          tag,
          validator,
          options,
          name,
          ..
        },
      type_ctx,
      validators_tokens,
      cel_rules,
      cel_checks,
    } = self;

    let proto_output_type = type_ctx.proto_field.output_proto_type();
    let proto_output_type_outer: Type = parse_quote! { #proto_output_type };

    field.change_type(proto_output_type_outer)?;

    let prost_attr = type_ctx.proto_field.as_prost_attr(*tag);
    let field_prost_attr: Attribute = parse_quote!(#prost_attr);
    field.inject_attr(field_prost_attr);

    let field_ident = field.ident()?;

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
      let field_validator = FieldValidatorExpr::new(type_ctx.proto_field, validator);

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

      let field_validator_tokens =
        type_ctx.validator_tokens(field_ident, field_context_tokens, &field_validator);

      validators_tokens.extend(field_validator_tokens);

      let new_cel_rules = field_validator.cel_rules_extractor_expr();

      cel_rules.push(new_cel_rules);

      let cel_check = field_validator.cel_check_expr();
      cel_checks.push(cel_check);

      let schema_expr = field_validator.schema_expr();

      quote! { Some(#schema_expr) }
    } else {
      quote! { None }
    };

    let field_type_tokens = type_ctx.proto_field.field_proto_type_tokens();
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
}
