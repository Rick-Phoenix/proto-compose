use crate::*;

impl FieldData {
  pub fn generate_proto_impls(
    &self,
    mut field: FieldOrVariant,
    validators_tokens: &mut Vec<TokenStream2>,
    consistency_checks: &mut Vec<TokenStream2>,
    tag_allocator: Option<&mut TagAllocator>,
  ) -> syn::Result<TokenStream2> {
    let FieldData {
      type_info,
      span: field_span,
      ident: field_ident,
      ident_str: field_ident_str,
      is_variant,
      tag,
      validator,
      options,
      proto_name,
      proto_field,
      ..
    } = self;

    let proto_output_type = proto_field.output_proto_type();
    let proto_output_type_outer: Type = parse_quote! { #proto_output_type };

    field.change_type(proto_output_type_outer)?;

    let tag = if let Some(tag) = tag {
      *tag
    } else if let Some(tag_allocator) = tag_allocator {
      tag_allocator
        .next_tag()
        .map_err(|e| error_with_span!(*field_span, "{e}"))?
    } else {
      bail_with_span!(*field_span, "Missing tag");
    };

    let prost_attr = proto_field.as_prost_attr(tag);
    let field_prost_attr: Attribute = parse_quote!(#prost_attr);
    field.inject_attr(field_prost_attr);

    if let ProtoField::Oneof {
      path: oneof_path, ..
    } = &proto_field
    {
      consistency_checks.push(quote! {
        (#field_ident_str, #oneof_path::check_validators_consistency())
      });

      validators_tokens.push(quote! {
        if let Some(oneof) = self.#field_ident.as_ref() {
          oneof.validate(parent_elements, violations);
        }
      });

      // For fields that are oneofs, we don't need to elaborate on the field type,
      // we delegate all the schema logic to the Oneof impl itself
      return Ok(quote! {
        ::prelude::MessageEntry::Oneof(<#oneof_path as ::prelude::ProtoOneof>::proto_schema())
      });
    }

    let validator_schema_tokens = if let Some(validator) = validator {
      let validator_target_type = proto_field.validator_target_type();

      let validator_static_ident =
        format_ident!("{}_VALIDATOR", ccase!(constant, &field_ident_str));

      let validator_expr = match validator {
        CallOrClosure::Call(call) => quote! { #call.build_validator() },

        CallOrClosure::Closure(closure) => {
          quote! { <#validator_target_type as ::prelude::ProtoValidator>::validator_from_closure(#closure) }
        }
      };

      let validator_name = proto_field.validator_name();

      let validator_static = quote! {
        static #validator_static_ident: LazyLock<#validator_name> = LazyLock::new(|| {
          #validator_expr
        });
      };

      let field_type = proto_field.descriptor_type_tokens();

      let field_context_tokens = quote! {
        ::prelude::FieldContext {
          proto_name: #proto_name,
          tag: #tag,
          field_type: #field_type,
          key_type: None,
          value_type: None,
          subscript: None,
          field_kind: Default::default(),
        }
      };

      let field_validator_tokens = generate_validator_tokens(
        &type_info.type_,
        *is_variant,
        field_ident,
        field_context_tokens,
        &validator_static_ident,
        validator_static,
      );

      validators_tokens.push(field_validator_tokens);

      consistency_checks.push(quote! {
        (#field_ident_str, #validator_expr.check_consistency())
      });

      quote! { Some(#validator_expr.into_schema()) }
    } else {
      quote! { None }
    };

    let field_type_tokens = proto_field.field_proto_type_tokens();
    let options_tokens = tokens_or_default!(options, quote! { vec![] });

    let output = match field {
      FieldOrVariant::Field(_) => {
        quote! {
          ::prelude::MessageEntry::Field(
            ::prelude::ProtoField {
              name: #proto_name.to_string(),
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
            name: #proto_name.to_string(),
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
