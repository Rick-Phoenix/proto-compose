use crate::*;

pub struct EnumVariantAttrs {
  pub name: String,
  pub options: TokensOr<TokenStream2>,
  pub deprecated: bool,
}

pub fn process_derive_enum_variants_attrs(
  enum_name: &str,
  variant_ident: &Ident,
  attrs: &[Attribute],
  no_prefix: bool,
) -> Result<EnumVariantAttrs, Error> {
  let mut options = TokensOr::<TokenStream2>::new(|_| quote! { ::prelude::vec![] });
  let mut name: Option<String> = None;
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
            "name" => {
              name = Some(meta.parse_value::<LitStr>()?.value());
            }
            _ => return Err(meta.error("Unknown attribute")),
          };

          Ok(())
        })?;
      }
      _ => {}
    }
  }

  let name = if let Some(name) = name {
    name
  } else {
    let plain_name = to_upper_snake_case(&variant_ident.to_string());

    if no_prefix {
      plain_name
    } else {
      let prefix = to_upper_snake_case(enum_name);
      format!("{prefix}_{plain_name}")
    }
  };

  Ok(EnumVariantAttrs {
    name,
    options,
    deprecated,
  })
}
