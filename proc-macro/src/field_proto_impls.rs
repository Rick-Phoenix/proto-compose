use crate::*;

pub struct FieldCtx<'a, 'field> {
  pub field: &'a mut FieldOrVariant<'field>,
  pub field_attrs: &'a FieldAttrs,
  pub type_ctx: &'a TypeContext<'a>,
  pub validators_tokens: &'a mut Vec<TokenStream2>,
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
      cel_checks.push(quote! {
        #oneof_path::check_cel_programs()
      });

      validators_tokens.push(quote! {
        if let Some(oneof) = self.#field_ident.as_ref() {
          oneof.validate(parent_elements).ok_or_push_violations(&mut violations)
        }
      });

      // Early return
      return Ok(quote! {
        ::prelude::MessageEntry::Oneof(<#oneof_path as ::prelude::ProtoOneof>::proto_schema())
      });
    }

    let is_oneof_variant = field.is_variant();

    let validator_schema_tokens = if let Some(validator) = validator {
      let validator_target_type = type_ctx.proto_field.validator_target_type();

      let validator_expr = match validator {
        CallOrClosure::Call(call) => quote! { #call.build_validator() },

        CallOrClosure::Closure(closure) => {
          quote! { <#validator_target_type as ::prelude::ProtoValidator>::validator_from_closure(#closure) }
        }
      };

      let field_type = type_ctx.proto_field.proto_kind_tokens();

      let field_context_tokens = quote! {
        ::prelude::FieldContext {
          proto_name: #name,
          tag: #tag,
          field_type: #field_type,
          key_type: None,
          value_type: None,
          subscript: None,
          field_kind: Default::default(),
        }
      };

      let field_validator_tokens = type_ctx.validator_tokens(
        is_oneof_variant,
        field_ident,
        field_context_tokens,
        &validator_expr,
      );

      validators_tokens.push(field_validator_tokens);

      cel_checks.push(quote! {
        #validator_expr.check_cel_programs()
      });

      quote! { Some(#validator_expr.into_schema()) }
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
