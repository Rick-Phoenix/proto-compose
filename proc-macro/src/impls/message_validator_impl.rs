use crate::*;

bool_enum!(pub UseFallback);

#[derive(Default)]
pub struct ValidatorsData {
  pub has_non_default_validators: bool,
  pub default_check_tokens: Vec<TokenStream2>,
}

pub fn field_validator_tokens(
  input_ident: &Ident,
  validators_data: &mut ValidatorsData,
  field_data: &FieldData,
  item_kind: ItemKind,
) -> Vec<TokenStream2> {
  let FieldData {
    ident,
    ident_str,
    tag,
    validators,
    proto_name,
    proto_field,
    type_info,
    ..
  } = field_data;

  let mut tokens: Vec<TokenStream2> = Vec::with_capacity(validators.validators.len());

  for v in validators {
    let ValidatorTokens {
      expr: validator_expr,
      kind,
      span,
    } = v;

    if !validators_data.has_non_default_validators {
      if kind.is_default() {
        if let Some(msg_info) = field_data.message_info()
          && msg_info
            .path
            .get_ident()
            .is_none_or(|i| i != input_ident)
        {
          let path = &msg_info.path;

          validators_data
            .default_check_tokens
            .push(if msg_info.boxed {
              quote! {
                <#path as ::prelude::ProtoValidation>::HAS_SHALLOW_VALIDATION
              }
            } else {
              quote! {
                <#path as ::prelude::ProtoValidation>::HAS_DEFAULT_VALIDATOR
              }
            });
        } else if let ProtoField::Oneof(oneof) = proto_field {
          let path = &oneof.path;

          validators_data.default_check_tokens.push(quote! {
            <#path as ::prelude::ProtoValidation>::HAS_DEFAULT_VALIDATOR
          });
        }
      } else {
        validators_data.has_non_default_validators = true;
      }
    }

    let argument = match item_kind {
      ItemKind::Oneof => quote_spanned! {*span=> Some(v) },
      ItemKind::Message => match type_info.type_.as_ref() {
        RustType::Option(inner) => {
          if inner.is_box() {
            quote_spanned! (*span=> self.#ident.as_deref())
          } else {
            quote_spanned! (*span=> self.#ident.as_ref())
          }
        }
        RustType::Box(_) => quote_spanned! (*span=> self.#ident.as_deref()),
        _ => {
          if matches!(
            proto_field,
            ProtoField::Single(ProtoType::Message(MessageInfo { .. })) | ProtoField::Oneof(_)
          ) {
            quote_spanned! (*span=> self.#ident.as_ref())
          } else {
            quote_spanned! (*span=> Some(&self.#ident))
          }
        }
      },
    };

    let field_type = field_data.descriptor_type_tokens();
    let validator_target_type = proto_field.validator_target_type(*span);

    let validate_args = if proto_field.is_oneof() {
      quote_spanned! {*span=>
        ctx,
        #argument
      }
    } else {
      quote_spanned! {*span=>
        ctx.with_field_context(
          ::prelude::FieldContext {
            name: #proto_name.into(),
            tag: #tag,
            field_type: #field_type,
            map_key_type: None,
            map_value_type: None,
            subscript: None,
            field_kind: Default::default(),
          }
        ),
        #argument
      }
    };

    let validator_call = if kind.should_be_cached() {
      let static_ident = format_ident!("{}_VALIDATOR", to_upper_snake_case(ident_str));
      let validator_name = field_data.validator_name();

      quote_spanned! {*span=>
        is_valid &= {
          static #static_ident: ::prelude::Lazy<#validator_name> = ::prelude::Lazy::new(|| {
            #validator_expr
          });

          ::prelude::Validator::<#validator_target_type>::validate_core(
            &*#static_ident,
            #validate_args
          )?
        };
      }
    } else {
      quote_spanned! {*span=>
        is_valid &= ::prelude::Validator::<#validator_target_type>::validate_core(
          &(#validator_expr),
          #validate_args
        )?;
      }
    };

    let output = if kind.is_default() {
      quote_spanned! {*span=>
        if <#validator_target_type as ::prelude::ProtoValidation>::HAS_DEFAULT_VALIDATOR {
          #validator_call
        }
      }
    } else {
      validator_call
    };

    tokens.push(output);
  }

  tokens
}

pub fn generate_message_validator(
  use_fallback: UseFallback,
  target_ident: &Ident,
  fields: &[FieldDataKind],
  top_level_validators: &Validators,
) -> TokenStream2 {
  let mut validators_data = ValidatorsData {
    has_non_default_validators: !top_level_validators.is_empty(),
    default_check_tokens: Vec::new(),
  };

  let validators_tokens = if *use_fallback {
    quote! { unimplemented!(); }
  } else {
    let top_level = top_level_validators.iter().enumerate().map(|(i, v)| {
      if v.kind.is_custom() {
        quote_spanned! {v.span=>
          is_valid &= ::prelude::Validator::<#target_ident>::validate_core(
            &(#v),
            ctx,
            Some(self)
          )?;
        }
      } else {
        let validator_static_ident = format_ident!("__VALIDATOR_{i}");

        quote_spanned! {v.span=>
          is_valid &= {
            static #validator_static_ident: ::prelude::Lazy<::prelude::CelValidator> = ::prelude::Lazy::new(|| {
              #v
            });

            ::prelude::Validator::<#target_ident>::validate_core(
              &*#validator_static_ident,
              ctx,
              Some(self)
            )?
          };
        }
      }
    });

    let field_validators = fields
      .iter()
      .filter_map(|d| d.as_normal())
      .flat_map(|d| {
        field_validator_tokens(target_ident, &mut validators_data, d, ItemKind::Message)
      });

    let top_level_tokens = quote! { #(#top_level)* };
    let field_validators_tokens = quote! { #(#field_validators)* };

    let has_field_validators = !field_validators_tokens.is_empty();
    let has_top_level_validators = !top_level_tokens.is_empty();

    if !has_top_level_validators && !has_field_validators {
      TokenStream2::new()
    } else {
      if !has_field_validators {
        quote! {
          #top_level_tokens
        }
      } else if !has_top_level_validators {
        quote! {
          #field_validators_tokens
        }
      } else {
        // Top level validators often include CEL validation which requires cloning
        // and heavier operations, so they should go last
        quote! {
          let top_level_field_context = ::core::mem::take(&mut ctx.field_context);

          #field_validators_tokens

          ctx.field_context = top_level_field_context;

          #top_level_tokens
        }
      }
    }
  };

  let has_validators = !validators_tokens.is_empty();

  let ValidatorsData {
    has_non_default_validators,
    default_check_tokens,
  } = validators_data;

  let inline_if_empty = (!has_validators).then(|| quote! { #[inline(always)] });

  let has_default_validator_tokens = if has_non_default_validators {
    quote! { true }
    // Means we only encountered boxed self for defaults, so it's false
  } else if default_check_tokens.is_empty() {
    quote! { false }
  } else {
    let mut tokens = TokenStream2::new();

    for (i, expr) in default_check_tokens.into_iter().enumerate() {
      if i != 0 {
        tokens.extend(quote! { || });
      }

      tokens.extend(expr);
    }

    tokens
  };

  quote! {
    impl ::prelude::ValidatedMessage for #target_ident {
      #[doc(hidden)]
      #inline_if_empty
      fn nested_validate(&self, ctx: &mut ::prelude::ValidationCtx) -> ::prelude::ValidatorResult {
        let mut is_valid = ::prelude::IsValid::Yes;

        #validators_tokens

        Ok(is_valid)
      }
    }

    impl ::prelude::ProtoValidation for #target_ident {
      #[doc(hidden)]
      type Target = Self;
      #[doc(hidden)]
      type Stored = Self;
      #[doc(hidden)]
      type Validator = ::prelude::MessageValidator;
      #[doc(hidden)]
      type Builder = ::prelude::MessageValidatorBuilder;

      type UniqueStore<'a>
        = ::prelude::LinearRefStore<'a, Self>
      where
        Self: 'a;

      const HAS_DEFAULT_VALIDATOR: bool = #has_default_validator_tokens;
      const HAS_SHALLOW_VALIDATION: bool = #has_non_default_validators;
    }
  }
}

impl MessageCtx<'_> {
  pub fn generate_validator(&self) -> TokenStream2 {
    let target_ident = self.proto_struct_ident();

    generate_message_validator(
      // For non-reflection implementations we don't skip fields if they don't have
      // validators, so having empty fields means an error occurred
      UseFallback::from(self.fields_data.is_empty()),
      target_ident,
      &self.fields_data,
      &self.message_attrs.validators,
    )
  }
}
