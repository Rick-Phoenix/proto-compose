use crate::*;

pub struct EnumVariantAttrs {
  pub name: String,
  pub options: Vec<Expr>,
}

pub fn process_derive_enum_variants_attrs(
  enum_name: &str,
  rust_variant_name: &Ident,
  attrs: &Vec<Attribute>,
  no_prefix: bool,
) -> Result<EnumVariantAttrs, Error> {
  let mut options: Vec<Expr> = Vec::new();
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
            _ => {}
          };
        }
        Meta::List(list) => {
          let ident = get_ident_or_continue!(list.path);

          match ident.as_str() {
            "options" => {
              let exprs = list.parse_args::<PunctuatedParser<Expr>>()?.inner;

              options = exprs.into_iter().collect();
            }
            _ => {}
          };
        }
        Meta::Path(_) => {}
      };
    }
  }

  let name = if let Some(name) = name {
    name
  } else {
    let plain_name = ccase!(constant, rust_variant_name.to_string());

    if no_prefix {
      plain_name
    } else {
      let prefix = ccase!(constant, enum_name);
      format!("{}_{}", prefix, plain_name)
    }
  };

  Ok(EnumVariantAttrs { options, name })
}
