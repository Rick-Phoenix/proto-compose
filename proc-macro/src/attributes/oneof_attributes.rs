use crate::*;

#[derive(Default)]
pub struct OneofAttrs {
  pub options: TokensOr<TokenStream2>,
  pub name: String,
  pub from_proto: Option<PathOrClosure>,
  pub into_proto: Option<PathOrClosure>,
  pub shadow_derives: Option<MetaList>,
  pub is_proxied: bool,
  pub no_auto_test: SkipAutoTest,
}

#[derive(Default)]
pub struct OneofMacroAttrs {
  pub is_proxied: bool,
  pub no_auto_test: SkipAutoTest,
}

impl OneofMacroAttrs {
  pub fn parse(macro_attrs: TokenStream2) -> syn::Result<Self> {
    let mut is_proxied = false;
    let mut no_auto_test = false;

    let macro_attrs_parser = syn::meta::parser(|meta| {
      let ident_str = meta.ident_str()?;

      match ident_str.as_str() {
        "proxied" => {
          is_proxied = true;
        }
        "no_auto_test" => {
          no_auto_test = true;
        }
        _ => return Err(meta.error("Unknown attribute")),
      };

      Ok(())
    });

    macro_attrs_parser.parse2(macro_attrs)?;

    Ok(Self {
      is_proxied,
      no_auto_test: no_auto_test.into(),
    })
  }
}

#[allow(clippy::needless_pass_by_value)]
pub fn process_oneof_attrs(
  enum_ident: &Ident,
  macro_attrs: OneofMacroAttrs,
  attrs: &[Attribute],
) -> Result<OneofAttrs, Error> {
  let mut options = TokensOr::<TokenStream2>::vec();
  let mut name: Option<String> = None;
  let mut from_proto: Option<PathOrClosure> = None;
  let mut into_proto: Option<PathOrClosure> = None;
  let mut shadow_derives: Option<MetaList> = None;

  parse_filtered_attrs(attrs, &["proto"], |meta| {
    let ident = meta.path.require_ident()?.to_string();

    match ident.as_str() {
      "derive" => {
        let list = meta.parse_list::<MetaList>()?;

        shadow_derives = Some(list);
      }
      "options" => {
        options.span = meta.input.span();
        options.set(meta.expr_value()?.into_token_stream());
      }
      "from_proto" => {
        from_proto = Some(meta.expr_value()?.as_path_or_closure()?);
      }
      "into_proto" => {
        into_proto = Some(meta.expr_value()?.as_path_or_closure()?);
      }
      "name" => name = Some(meta.expr_value()?.as_string()?),
      _ => return Err(meta.error("Unknown attribute")),
    };

    Ok(())
  })?;

  Ok(OneofAttrs {
    options,
    name: name.unwrap_or_else(|| to_snake_case(&enum_ident.to_string())),
    from_proto,
    into_proto,
    shadow_derives,
    is_proxied: macro_attrs.is_proxied,
    no_auto_test: macro_attrs.no_auto_test,
  })
}
