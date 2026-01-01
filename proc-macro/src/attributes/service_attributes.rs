use crate::*;

pub struct ServiceOrHandlerAttrs {
  pub options: TokensOr<TokenStream2>,
}

pub fn process_service_or_handler_attrs(
  attrs: &[Attribute],
) -> Result<ServiceOrHandlerAttrs, Error> {
  let mut options = TokensOr::<TokenStream2>::new(|| quote! { vec![] });

  parse_filtered_attrs(attrs, &["proto"], |meta| {
    let ident_str = meta.ident_str()?;

    match ident_str.as_str() {
      "options" => {
        options.set(meta.expr_value()?.into_token_stream());
      }
      _ => return Err(meta.error("Unknown attribute")),
    };

    Ok(())
  })?;

  Ok(ServiceOrHandlerAttrs { options })
}
