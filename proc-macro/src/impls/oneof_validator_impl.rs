use crate::*;

pub struct OneofValidatorImplCtx<'a> {
  pub oneof_ident: &'a Ident,
  pub validators_tokens: Vec<TokenStream2>,
}

pub fn impl_oneof_validator(ctx: OneofValidatorImplCtx) -> TokenStream2 {
  let OneofValidatorImplCtx {
    oneof_ident,
    validators_tokens,
  } = ctx;

  quote! {
    #[allow(clippy::ptr_arg)]
    impl #oneof_ident {
      pub fn validate(&self, parent_elements: &mut Vec<FieldPathElement>) -> Result<(), Violations> {
        let mut violations = Violations::new();

        match self {
          #(#validators_tokens,)*
          _ => {}
        }

        if violations.is_empty() {
          Ok(())
        } else {
          Err(violations)
        }
      }
    }
  }
}
