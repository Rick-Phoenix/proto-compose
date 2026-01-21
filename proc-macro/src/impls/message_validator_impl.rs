use crate::*;

bool_enum!(pub UseFallback);

pub fn field_validator_tokens(field_data: &FieldData, item_kind: ItemKind) -> Vec<TokenStream2> {
  let FieldData {
    ident,
    ident_str,
    tag,
    validators,
    proto_name,
    proto_field,
    span,
    type_info,
    ..
  } = field_data;

  let mut tokens: Vec<TokenStream2> = Vec::new();

  if let ProtoField::Oneof(OneofInfo { required, .. }) = proto_field {
    tokens.push(quote_spanned! {*span=>
      is_valid &= ::prelude::validate_oneof(self.#ident.as_ref(), ctx, #required)?;
    });
  } else {
    tokens = validators.iter().map(|validator| {
      let ValidatorTokens {
        expr: validator_expr,
        span,
        kind,
        ..
      } = validator;

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
            if let ProtoField::Single(ProtoType::Message(MessageInfo { .. })) = proto_field {
              quote_spanned! (*span=> self.#ident.as_ref())
            } else {
              quote_spanned! (*span=> Some(&self.#ident))
            }
          }
        },
      };

      let field_type = field_data.descriptor_type_tokens();
      let validator_target_type = proto_field.validator_target_type(*span);

      let validate_args = quote! {
        ctx.with_field_context(
          ::prelude::FieldContext {
            proto_name: #proto_name,
            tag: #tag,
            field_type: #field_type,
            map_key_type: None,
            map_value_type: None,
            subscript: None,
            field_kind: Default::default(),
          }
        ),
        #argument
      };

      if kind.is_custom() {
        quote_spanned! {*span=>
          is_valid &= ::prelude::Validator::<#validator_target_type>::validate_core(
            &(#validator_expr),
            #validate_args
          )?;
        }
      } else {
        let validator_static_ident = format_ident!("{}_VALIDATOR", to_upper_snake_case(ident_str));
        let validator_name = field_data.validator_name();

        if kind.is_default() {
          quote_spanned! {*span=>
            if <#validator_target_type as ::prelude::ProtoValidator>::HAS_DEFAULT_VALIDATOR {
              static #validator_static_ident: ::prelude::Lazy<#validator_name> = ::prelude::Lazy::new(|| {
                #validator_expr
              });

              is_valid &= ::prelude::Validator::<#validator_target_type>::validate_core(
                &*#validator_static_ident,
                #validate_args
              )?;
            }
          }
        } else {
          quote_spanned! {*span=>
            is_valid &= {
              static #validator_static_ident: ::prelude::Lazy<#validator_name> = ::prelude::Lazy::new(|| {
                #validator_expr
              });

              ::prelude::Validator::<#validator_target_type>::validate_core(
                &*#validator_static_ident,
                #validate_args
              )?
            };
          }
        }
      }
    }).collect();
  }

  tokens
}

pub fn generate_message_validator(
  use_fallback: UseFallback,
  target_ident: &Ident,
  fields: &[FieldDataKind],
  top_level_validators: &Validators,
) -> TokenStream2 {
  let mut maybe_default_validators = 0;
  let mut non_default_validators = 0;

  let validators_tokens = if *use_fallback {
    quote! { unimplemented!(); }
  } else {
    let top_level = top_level_validators.iter().enumerate().map(|(i, v)| {
      non_default_validators += 1;

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
      .flat_map(|d| field_validator_tokens(d, ItemKind::Message));

    let all_validators = top_level.chain(field_validators);

    quote! { #(#all_validators)* }
  };

  // Validators will always be populated if a field is marked
  // as a message (or vec/map of messages), or as a oneof,
  // because we cannot know if it has validators of its own.
  let has_validators = !validators_tokens.is_empty();

  let inline_if_empty = (!has_validators).then(|| quote! { #[inline(always)] });

  for v in fields
    .iter()
    .filter_map(|f| f.as_normal())
    .flat_map(|d| &d.validators)
  {
    if v.kind.is_default() {
      maybe_default_validators += 1;
    } else {
      non_default_validators += 1;
    }
  }

  let total_validators = maybe_default_validators + non_default_validators;

  let has_default_validator_tokens = if total_validators == 0 {
    quote! { false }
  } else if non_default_validators > 0 {
    quote! { true }
  } else {
    let message_paths = fields
      .iter()
      .filter_map(|f| f.as_normal())
      .filter_map(|f| f.message_path())
      .filter(|p| p.get_ident().is_none_or(|i| i != target_ident));

    let mut has_default_validator_tokens = TokenStream2::new();

    for (i, path) in message_paths.enumerate() {
      if i != 0 {
        has_default_validator_tokens.extend(quote! { && });
      }

      has_default_validator_tokens
        .extend(quote! { <#path as ::prelude::ProtoValidator>::HAS_DEFAULT_VALIDATOR });
    }

    if has_default_validator_tokens.is_empty() {
      has_default_validator_tokens = quote! { false };
    }

    has_default_validator_tokens
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

    impl ::prelude::ProtoValidator for #target_ident {
      #[doc(hidden)]
      type Target = Self;
      #[doc(hidden)]
      type Validator = ::prelude::MessageValidator;
      #[doc(hidden)]
      type Builder = ::prelude::MessageValidatorBuilder;

      type UniqueStore<'a>
        = ::prelude::LinearRefStore<'a, Self>
      where
        Self: 'a;

      const HAS_DEFAULT_VALIDATOR: bool = #has_default_validator_tokens;
    }
  }
}

impl MessageCtx<'_> {
  pub fn generate_validator(&self) -> TokenStream2 {
    let target_ident = self.proto_struct_ident();

    generate_message_validator(
      // For non-reflection implementations we don't skip fields if they don't have
      // validators, so empty fields = an error occurred
      self.fields_data.is_empty().into(),
      target_ident,
      &self.fields_data,
      &self.message_attrs.validators,
    )
  }
}
