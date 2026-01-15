use crate::*;

pub struct ServiceOrHandlerAttrs {
  pub options: TokensOr<TokenStream2>,
  pub deprecated: bool,
}

pub fn process_service_or_handler_attrs(
  attrs: &[Attribute],
) -> Result<ServiceOrHandlerAttrs, Error> {
  let mut options = TokensOr::<TokenStream2>::new(|_| quote! { ::prelude::vec![] });
  let mut deprecated = false;

  for attr in attrs {
    let ident = if let Some(ident) = attr.path().get_ident() {
      ident.to_string()
    } else {
      continue;
    };

    match ident.as_str() {
      "deprecated" => {
        deprecated = true;
      }
      "proto" => {
        attr.parse_nested_meta(|meta| {
          let ident_str = meta.ident_str()?;

          match ident_str.as_str() {
            "deprecated" => {
              let boolean = meta.parse_value::<LitBool>()?;

              deprecated = boolean.value();
            }
            "options" => {
              options.span = meta.input.span();
              options.set(meta.expr_value()?.into_token_stream());
            }
            _ => return Err(meta.error("Unknown attribute")),
          };

          Ok(())
        })?;
      }
      _ => {}
    }
  }

  Ok(ServiceOrHandlerAttrs {
    options,
    deprecated,
  })
}
