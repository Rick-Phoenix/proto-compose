use syn_utils::filter_attributes;

use crate::*;

pub struct EnumAttrs {
  pub reserved_names: Vec<String>,
  pub reserved_numbers: ReservedNumbers,
  pub options: Option<Expr>,
  pub parent_message: Option<Ident>,
  pub name: String,
  pub no_prefix: bool,
  pub extern_path: Option<String>,
}

pub fn process_derive_enum_attrs(
  enum_ident: &Ident,
  attrs: &[Attribute],
) -> Result<EnumAttrs, Error> {
  let mut reserved_names: Vec<String> = Vec::new();
  let mut reserved_numbers = ReservedNumbers::default();
  let mut options: Option<Expr> = None;
  let mut proto_name: Option<String> = None;
  let mut no_prefix = false;
  let mut parent_message: Option<Ident> = None;
  let mut extern_path: Option<String> = None;

  for arg in filter_attributes(attrs, &["proto"])? {
    match arg {
      Meta::List(list) => {
        let ident = list.path.require_ident()?.to_string();

        match ident.as_str() {
          "reserved_names" => {
            let names = list.parse_args::<StringList>()?;

            reserved_names = names.list;
          }
          "reserved_numbers" => {
            let numbers = list.parse_args::<ReservedNumbers>()?;

            reserved_numbers = numbers;
          }

          _ => bail!(list, "Unknown attribute `{ident}`"),
        };
      }
      Meta::NameValue(nv) => {
        let ident = nv.path.require_ident()?.to_string();

        match ident.as_str() {
          "extern_path" => {
            extern_path = Some(nv.value.as_string()?);
          }
          "parent_message" => {
            parent_message = Some(nv.value.as_path()?.require_ident()?.clone());
          }
          "options" => {
            options = Some(nv.value);
          }
          "name" => {
            proto_name = Some(nv.value.as_string()?);
          }
          _ => bail!(nv.path, "Unknown attribute `{ident}`"),
        };
      }
      Meta::Path(path) => {
        let ident = path.require_ident()?.to_string();

        match ident.as_str() {
          "no_prefix" => no_prefix = true,
          _ => bail!(path, "Unknown attribute `{ident}`"),
        };
      }
    }
  }

  let name = proto_name.unwrap_or_else(|| ccase!(pascal, enum_ident.to_string()));

  Ok(EnumAttrs {
    extern_path,
    reserved_names,
    reserved_numbers,
    options,
    name,
    no_prefix,
    parent_message,
  })
}
