use crate::*;

bool_enum!(pub UseFallback);

pub fn field_validator_tokens(field_data: &FieldData, item_kind: ItemKind) -> Option<TokenStream2> {
  let FieldData {
    ident,
    ident_str,
    tag,
    validator,
    proto_name,
    proto_field,
    span,
    type_info,
    ..
  } = field_data;

  if let ProtoField::Oneof(OneofInfo { required, .. }) = proto_field {
    Some(if *required {
      quote_spanned! {*span=>
        match self.#ident.as_ref() {
          Some(oneof) => {
            if !::prelude::ValidatedOneof::validate(oneof, ctx) {
              is_valid = false;
            }
          },
          None => {
            ctx.violations.add_required_oneof_violation(&ctx.parent_elements);
            is_valid = false;
          }
        }
      }
    } else {
      quote_spanned! {*span=>
        if let Some(oneof) = self.#ident.as_ref() {
          if !::prelude::ValidatedOneof::validate(oneof, ctx) {
            is_valid = false;
          }
        }
      }
    })
  } else {
    validator.as_ref().map(|validator| {
      let ValidatorTokens {
        expr: validator_expr,
        span,
        ..
      } = validator;

      let validator_static_ident = format_ident!("{}_VALIDATOR", to_upper_snake_case(ident_str));
      let validator_name = field_data.validator_name();
      let field_type = field_data.descriptor_type_tokens();

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

      let validate_method = {
        let validator_name = field_data.validator_name();
        let validator_target_type = proto_field.validator_target_type(*span);

        quote_spanned! {*span=>
          <#validator_name as ::prelude::Validator<#validator_target_type>>::validate_core(
            &#validator_static_ident,
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
          )
        }
      };

      quote_spanned! {*span=>
        static #validator_static_ident: ::prelude::Lazy<#validator_name> = ::prelude::Lazy::new(|| {
          #validator_expr
        });

        #validate_method
      }
    })
  }
}

pub fn generate_message_validator(
  use_fallback: UseFallback,
  target_ident: &Ident,
  fields: &[FieldDataKind],
  top_level_cel_rules: &TokenStream2,
) -> TokenStream2 {
  let validators_tokens = if *use_fallback {
    quote! { unimplemented!(); }
  } else {
    let tokens = fields
      .iter()
      .filter_map(|d| d.as_normal())
      .filter_map(|data| {
        field_validator_tokens(data, ItemKind::Message).map(|validator| {
          let span = data.span;

          let check = if data.proto_field.is_oneof() {
            validator
          } else {
            quote_spanned! {span=>
              if !{ #validator } {
                is_valid = false;
              }
            }
          };

          quote_spanned! {span=>
            #check

            if !is_valid && ctx.fail_fast {
              return false;
            }
          }
        })
      });

    quote! { #(#tokens)* }
  };

  let has_cel_rules = !top_level_cel_rules.is_empty();

  let cel_methods = if has_cel_rules {
    quote_spanned! {top_level_cel_rules.span()=>
      #[inline]
      #[allow(clippy::iter_on_single_items)]
      fn cel_rules() -> &'static [::prelude::CelProgram] {
        static PROGRAMS: ::prelude::Lazy<::prelude::Box<[::prelude::CelProgram]>> = ::prelude::Lazy::new(|| {
          let programs: ::prelude::Vec<::prelude::CelProgram> = #top_level_cel_rules.into_iter().collect();
          programs.into_boxed_slice()
        });

        &PROGRAMS
      }
    }
  } else {
    // `cel_rules` will use the default impl
    quote! {
      #[inline(always)]
      fn validate_cel(&self, _: &mut ::prelude::ValidationCtx) -> bool {
        true
      }
    }
  };

  // Validators will always be populated if a field is marked
  // as a message (or vec/map of messages), or as a oneof,
  // because we cannot know if it has validators of its own.
  let has_validators = !validators_tokens.is_empty() || has_cel_rules;

  let validator_impl = if has_validators {
    quote! {
      impl ::prelude::ValidatedMessage for #target_ident {
        #cel_methods

        #[doc(hidden)]
        fn nested_validate(&self, ctx: &mut ::prelude::ValidationCtx) -> bool {
          let mut is_valid = true;

          ::prelude::ValidatedMessage::validate_cel(self, ctx);

          #validators_tokens

          is_valid
        }
      }
    }
  } else {
    quote! {
      impl ::prelude::ValidatedMessage for #target_ident {
        #cel_methods

        #[inline(always)]
        fn validate(&self) -> Result<(), ::prelude::ViolationsAcc> {
          Ok(())
        }

        #[doc(hidden)]
        #[inline(always)]
        fn nested_validate(&self, ctx: &mut ::prelude::ValidationCtx) -> bool {
          true
        }
      }
    }
  };

  // The default impl will be used otherwise
  let default_validator_method = has_validators.then(|| {
    quote! {
      #[doc(hidden)]
      #[inline]
      fn default_validator() -> Option<Self::Validator> {
        Some(::prelude::MessageValidator::default())
      }
    }
  });

  quote! {
    #validator_impl

    impl ::prelude::ProtoValidator for #target_ident {
      #[doc(hidden)]
      type Target = Self;
      #[doc(hidden)]
      type Validator = ::prelude::MessageValidator<Self>;
      #[doc(hidden)]
      type Builder = ::prelude::MessageValidatorBuilder<Self>;

      type UniqueStore<'a>
        = ::prelude::LinearRefStore<'a, Self>
      where
        Self: 'a;

      #default_validator_method
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
      &self.message_attrs.cel_rules,
    )
  }
}
