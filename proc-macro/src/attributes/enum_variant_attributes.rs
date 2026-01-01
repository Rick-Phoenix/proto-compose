use crate::*;

pub struct EnumVariantAttrs {
  pub name: String,
  pub options: TokensOr<TokenStream2>,
}

pub fn process_derive_enum_variants_attrs(
  enum_name: &str,
  variant_ident: &Ident,
  attrs: &[Attribute],
  no_prefix: bool,
) -> Result<EnumVariantAttrs, Error> {
  let mut options = TokensOr::<TokenStream2>::new(|| quote! { vec![] });
  let mut name: Option<String> = None;

  parse_filtered_attrs(attrs, &["proto"], |meta| {
    let ident_str = meta.ident_str()?;

    match ident_str.as_str() {
      "options" => {
        options.set(meta.expr_value()?.into_token_stream());
      }
      "name" => {
        name = Some(meta.parse_value::<LitStr>()?.value());
      }
      _ => return Err(meta.error("Unknown attribute")),
    };

    Ok(())
  })?;

  let name = if let Some(name) = name {
    name
  } else {
    let plain_name = ccase!(constant, variant_ident.to_string());

    if no_prefix {
      plain_name
    } else {
      let prefix = ccase!(constant, enum_name);
      format!("{}_{}", prefix, plain_name)
    }
  };

  Ok(EnumVariantAttrs { options, name })
}
