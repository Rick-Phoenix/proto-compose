use crate::*;

#[derive(Default)]
pub struct OneofAttrs {
  pub options: TokensOr<TokenStream2>,
  pub name: String,
  pub from_proto: Option<PathOrClosure>,
  pub into_proto: Option<PathOrClosure>,
  pub proto_derives: Vec<Path>,
  pub is_proxied: bool,
  pub auto_tests: AutoTests,
  pub validators: Validators,
}

#[derive(Default)]
pub struct OneofMacroAttrs {
  pub is_proxied: bool,
}

impl OneofMacroAttrs {
  pub fn parse(macro_attrs: TokenStream2) -> syn::Result<Self> {
    let mut is_proxied = false;

    let macro_attrs_parser = syn::meta::parser(|meta| {
      let ident_str = meta.ident_str()?;

      match ident_str.as_str() {
        "proxied" => {
          is_proxied = true;
        }
        _ => return Err(meta.error("Unknown attribute")),
      };

      Ok(())
    });

    macro_attrs_parser.parse2(macro_attrs)?;

    Ok(Self { is_proxied })
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
  let mut shadow_derives: Vec<Path> = Vec::new();
  let mut auto_tests = AutoTests::default();
  let mut validators = Validators::default();

  parse_filtered_attrs(attrs, &["proto"], |meta| {
    let ident = meta.path.require_ident()?.to_string();

    match ident.as_str() {
      "validate" => {
        validators = meta.parse_value::<Validators>()?;
      }
      "skip_checks" => {
        auto_tests = AutoTests::parse(&meta)?;
      }
      "derive" => {
        shadow_derives = meta.parse_list::<PathList>()?.list;
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

  for validator in &validators {
    if validator.kind.is_closure() {
      bail_with_span!(validator.span, "Closures are not supported for oneofs");
    }
  }

  Ok(OneofAttrs {
    options,
    name: name.unwrap_or_else(|| to_snake_case(&enum_ident.to_string())),
    from_proto,
    into_proto,
    proto_derives: shadow_derives,
    is_proxied: macro_attrs.is_proxied,
    auto_tests,
    validators,
  })
}
