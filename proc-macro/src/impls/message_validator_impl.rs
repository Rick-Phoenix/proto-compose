use crate::*;

bool_enum!(pub UseFallback);

pub struct ValidatorsData<'a> {
  pub non_default_validators: usize,
  pub maybe_default_validators: usize,
  pub paths_to_check: Vec<&'a Path>,
}

pub fn field_validator_tokens<'a>(
  input_ident: &Ident,
  validators_data: &mut ValidatorsData<'a>,
  field_data: &'a FieldData,
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

    if kind.is_default() {
      validators_data.maybe_default_validators += 1;

      if let Some(msg_info) = field_data.message_info()
        && !msg_info.boxed
        && msg_info
          .path
          .get_ident()
          .is_none_or(|i| i != input_ident)
      {
        validators_data
          .paths_to_check
          .push(&msg_info.path);
      } else if let ProtoField::Oneof(oneof) = proto_field {
        validators_data.paths_to_check.push(&oneof.path);
      }
    } else {
      validators_data.non_default_validators += 1;
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
        if <#validator_target_type as ::prelude::ProtoValidator>::HAS_DEFAULT_VALIDATOR {
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
    non_default_validators: top_level_validators.len(),
    maybe_default_validators: 0,
    paths_to_check: vec![],
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

    let all_validators = top_level.chain(field_validators);

    quote! { #(#all_validators)* }
  };

  let has_validators =
    validators_data.maybe_default_validators + validators_data.non_default_validators != 0;

  let inline_if_empty = (!has_validators).then(|| quote! { #[inline(always)] });

  let has_default_validator_tokens = if !has_validators {
    quote! { false }
  } else if validators_data.non_default_validators > 0 {
    quote! { true }
  } else {
    let mut has_default_validator_tokens = TokenStream2::new();

    for (i, path) in validators_data.paths_to_check.iter().enumerate() {
      if i != 0 {
        has_default_validator_tokens.extend(quote! { && });
      }

      has_default_validator_tokens
        .extend(quote! { <#path as ::prelude::ProtoValidator>::HAS_DEFAULT_VALIDATOR });
    }

    // This can still happen if the only element in the paths_to_check
    // is this same message, which was boxed. In that case,
    // if we got to this point, non_default_validators is 0,
    // so this should be false
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
