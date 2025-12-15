use syn_utils::filter_attributes;

use crate::*;

pub struct ServiceOrHandlerAttrs {
  pub name: String,
  pub options: Option<Expr>,
  pub package: Option<String>,
}

pub fn process_service_or_handler_attrs(
  ident: &Ident,
  attrs: &[Attribute],
) -> Result<ServiceOrHandlerAttrs, Error> {
  let mut options: Option<Expr> = None;
  let mut name: Option<String> = None;
  let mut package: Option<String> = None;

  for arg in filter_attributes(attrs, &["proto"])? {
    match arg {
      Meta::NameValue(nv) => {
        let ident = nv.path.require_ident()?.to_string();

        match ident.as_str() {
          "options" => {
            options = Some(nv.value);
          }
          "name" => {
            name = Some(nv.value.as_string()?);
          }

          "package" => {
            package = Some(nv.value.as_string()?);
          }

          _ => bail!(nv.path, "Unknown attribute `{ident}`"),
        };
      }
      Meta::List(_) => {}
      Meta::Path(_) => {}
    }
  }

  let name = name.unwrap_or_else(|| ccase!(pascal, ident.to_string()));

  Ok(ServiceOrHandlerAttrs {
    package,
    name,
    options,
  })
}
