use crate::*;

pub struct ServiceOrHandlerAttrs {
  pub name: String,
  pub options: Option<Expr>,
  pub package: Option<String>,
}

pub fn process_service_or_handler_attrs(
  ident: &Ident,
  attrs: &Vec<Attribute>,
) -> Result<ServiceOrHandlerAttrs, Error> {
  let mut options: Option<Expr> = None;
  let mut name: Option<String> = None;
  let mut package: Option<String> = None;

  for attr in attrs {
    if !attr.path().is_ident("proto") {
      continue;
    }

    let args = attr.parse_args::<PunctuatedParser<Meta>>()?;

    for arg in args.inner {
      match arg {
        Meta::NameValue(nv) => {
          let ident = nv.path.require_ident()?.to_string();

          match ident.as_str() {
            "options" => {
              options = Some(nv.value);
            }
            "name" => {
              name = Some(extract_string_lit(&nv.value)?);
            }

            "package" => {
              package = Some(extract_string_lit(&nv.value)?);
            }

            _ => bail!(nv.path, format!("Unknown attribute `{ident}`")),
          };
        }
        Meta::List(_) => {}
        Meta::Path(_) => {}
      }
    }
  }

  let name = name.unwrap_or_else(|| ccase!(pascal, ident.to_string()));

  Ok(ServiceOrHandlerAttrs {
    package,
    name,
    options,
  })
}
