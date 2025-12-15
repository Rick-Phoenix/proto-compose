use syn_utils::filter_attributes;

use crate::*;

pub struct EnumVariantAttrs {
  pub name: String,
  pub options: Option<Expr>,
}

pub fn process_derive_enum_variants_attrs(
  enum_name: &str,
  variant_ident: &Ident,
  attrs: &[Attribute],
  no_prefix: bool,
) -> Result<EnumVariantAttrs, Error> {
  let mut options: Option<Expr> = None;
  let mut name: Option<String> = None;

  for arg in filter_attributes(attrs, &["proto"])? {
    match arg {
      Meta::NameValue(nv) => {
        let ident = nv.path.require_ident()?.to_string();

        match ident.as_str() {
          "name" => {
            name = Some(nv.value.as_string()?);
          }
          "options" => {
            options = Some(nv.value);
          }
          _ => bail!(nv.path, "Unknown attribute `{ident}`"),
        };
      }
      Meta::List(_) => {}
      Meta::Path(_) => {}
    };
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
