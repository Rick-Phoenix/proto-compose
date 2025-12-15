use syn_utils::filter_attributes;

use crate::*;

pub struct EnumAttrs {
  pub reserved_names: Vec<String>,
  pub reserved_numbers: ReservedNumbers,
  pub options: Option<Expr>,
  pub name: String,
  pub file: String,
  pub package: String,
  pub full_name: String,
  pub no_prefix: bool,
  pub backend: Backend,
}

pub fn process_derive_enum_attrs(
  enum_ident: &Ident,
  attrs: &[Attribute],
) -> Result<EnumAttrs, Error> {
  let mut reserved_names: Vec<String> = Vec::new();
  let mut reserved_numbers = ReservedNumbers::default();
  let mut options: Option<Expr> = None;
  let mut proto_name: Option<String> = None;
  let mut full_name: Option<String> = None;
  let mut file: Option<String> = None;
  let mut package: Option<String> = None;
  let mut no_prefix = false;
  let mut backend = Backend::default();

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
          "backend" => {
            backend = Backend::from_expr(&nv.value)?;
          }
          "options" => {
            options = Some(nv.value);
          }
          "name" => {
            proto_name = Some(nv.value.as_string()?);
          }
          "full_name" => {
            full_name = Some(nv.value.as_string()?);
          }
          "package" => {
            package = Some(nv.value.as_string()?);
          }
          "file" => {
            file = Some(nv.value.as_string()?);
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
  let full_name = full_name.unwrap_or_else(|| name.clone());

  let file = file.ok_or(error_call_site!(
    r#"`file` attribute is missing. Use the `proto_module` macro on the surrounding module or set it manually with #[proto(file = "my_file.proto")]"#
  ))?;
  let package = package.ok_or(error_call_site!(r#"`package` attribute is missing. Use the `proto_module` macro on the surrounding module or set it manually with #[proto(package = "mypackage.v1")]"#))?;

  Ok(EnumAttrs {
    reserved_names,
    reserved_numbers,
    options,
    name,
    file,
    package,
    full_name,
    no_prefix,
    backend,
  })
}
