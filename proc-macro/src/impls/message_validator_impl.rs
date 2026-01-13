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
          Some(oneof) => ::prelude::ValidatedOneof::validate(oneof, parent_elements, violations),
          None => violations.add_required_oneof_violation(parent_elements)
        };
      }
    } else {
      quote_spanned! {*span=>
        if let Some(oneof) = self.#ident.as_ref() {
          ::prelude::ValidatedOneof::validate(oneof, parent_elements, violations);
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
        ItemKind::Oneof => quote! { Some(v) },
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

      quote_spanned! {*span=>
        static #validator_static_ident: LazyLock<#validator_name> = LazyLock::new(|| {
          #validator_expr
        });

        #validator_static_ident.validate(
          &mut ::prelude::ValidationCtx {
            field_context: ::prelude::FieldContext {
              proto_name: #proto_name,
              tag: #tag,
              field_type: #field_type,
              map_key_type: None,
              map_value_type: None,
              subscript: None,
              field_kind: Default::default(),
            },
            parent_elements,
            violations
          },
          #argument
        );
      }
    })
  }
}

pub fn generate_message_validator(
  use_fallback: UseFallback,
  target_ident: &Ident,
  fields: &[FieldDataKind],
  top_level_cel_rules: &IterTokensOr<TokenStream2>,
) -> TokenStream2 {
  let validators_tokens = if *use_fallback {
    quote! { unimplemented!(); }
  } else {
    let tokens = fields
      .iter()
      .filter_map(|d| d.as_normal())
      .filter_map(|data| field_validator_tokens(data, ItemKind::Message));

    quote! { #(#tokens)* }
  };

  let has_cel_rules = !top_level_cel_rules.is_empty();

  let cel_rules_method = has_cel_rules.then(|| {
      quote_spanned! {top_level_cel_rules.span()=>
        #[inline]
        fn cel_rules() -> &'static [::prelude::CelProgram] {
          static PROGRAMS: std::sync::LazyLock<Vec<::prelude::CelProgram>> = std::sync::LazyLock::new(|| {
            #top_level_cel_rules
          });

          &PROGRAMS
        }
      }
    });

  let cel_rules_call = has_cel_rules.then(|| {
    quote_spanned! {top_level_cel_rules.span()=>
      ::prelude::ValidatedMessage::validate_cel(self, field_context, parent_elements, violations);
    }
  });

  quote! {
    #[allow(clippy::ptr_arg)]
    impl #target_ident {
      #[doc(hidden)]
      fn __validate_internal(&self, field_context: Option<&::prelude::FieldContext>, parent_elements: &mut Vec<::prelude::FieldPathElement>, violations: &mut ::prelude::ViolationsAcc) {
        #cel_rules_call

        #validators_tokens
      }
    }

    impl ::prelude::ValidatedMessage for #target_ident {
      #cel_rules_method

      fn validate(&self) -> Result<(), ::prelude::Violations> {
        let mut violations = ::prelude::ViolationsAcc::new();

        self.__validate_internal(None, &mut vec![], &mut violations);

        if violations.is_empty() {
          Ok(())
        } else {
          Err(violations.to_vec())
        }
      }

      #[doc(hidden)]
      #[inline]
      fn nested_validate(&self, ctx: &mut ::prelude::ValidationCtx) {
        self.__validate_internal(Some(&ctx.field_context), ctx.parent_elements, ctx.violations)
      }
    }

    impl ::prelude::ProtoValidator for #target_ident {
      #[doc(hidden)]
      type Target = Self;
      #[doc(hidden)]
      type Validator = ::prelude::MessageValidator<Self>;
      #[doc(hidden)]
      type Builder = ::prelude::MessageValidatorBuilder<Self>;

      #[doc(hidden)]
      #[inline]
      fn default_validator() -> Option<Self::Validator> {
        Some(::prelude::MessageValidator::default())
      }
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
