use crate::*;

pub struct EnumVariantAttrs {
  pub name: String,
  pub options: Option<Expr>,
}

pub fn process_derive_enum_variants_attrs(
  enum_name: &str,
  variant_ident: &Ident,
  attrs: &Vec<Attribute>,
  no_prefix: bool,
) -> Result<EnumVariantAttrs, Error> {
  let mut options: Option<Expr> = None;
  let mut name: Option<String> = None;

  for attr in attrs {
    if !attr.path().is_ident("proto") {
      continue;
    }

    let args = attr.parse_args::<PunctuatedParser<Meta>>()?;

    for meta in args.inner {
      match meta {
        Meta::NameValue(nv) => {
          let ident = get_ident_or_continue!(nv.path);

          match ident.as_str() {
            "name" => {
              name = Some(extract_string_lit(&nv.value)?);
            }
            "options" => {
              options = Some(nv.value);
            }
            _ => {}
          };
        }
        Meta::List(_) => {}
        Meta::Path(_) => {}
      };
    }
  }

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
