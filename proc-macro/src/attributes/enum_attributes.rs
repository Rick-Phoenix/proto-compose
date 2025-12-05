use crate::*;

pub struct EnumAttrs {
  pub reserved_names: ReservedNames,
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
  attrs: &Vec<Attribute>,
) -> Result<EnumAttrs, Error> {
  let mut reserved_names = ReservedNames::default();
  let mut reserved_numbers = ReservedNumbers::default();
  let mut options: Option<Expr> = None;
  let mut proto_name: Option<String> = None;
  let mut full_name: Option<String> = None;
  let mut file: Option<String> = None;
  let mut package: Option<String> = None;
  let mut no_prefix = false;
  let mut backend = Backend::default();

  for attr in attrs {
    if !attr.path().is_ident("proto") {
      continue;
    }

    let args = attr.parse_args::<PunctuatedParser<Meta>>()?;

    for arg in args.inner {
      match arg {
        Meta::List(list) => {
          let ident = list.path.require_ident()?.to_string();

          match ident.as_str() {
            "reserved_names" => {
              let names = list.parse_args::<StringList>()?;

              reserved_names = ReservedNames::List(names.list);
            }
            "reserved_numbers" => {
              let numbers = list.parse_args::<ReservedNumbers>()?;

              reserved_numbers = numbers;
            }

            _ => bail!(list, format!("Unknown attribute `{ident}`")),
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
              proto_name = Some(extract_string_lit(&nv.value)?);
            }
            "reserved_names" => {
              reserved_names = ReservedNames::Expr(nv.value);
            }
            "full_name" => {
              full_name = Some(extract_string_lit(&nv.value)?);
            }
            "package" => {
              package = Some(extract_string_lit(&nv.value)?);
            }
            "file" => {
              file = Some(extract_string_lit(&nv.value)?);
            }
            _ => bail!(nv.path, format!("Unknown attribute `{ident}`")),
          };
        }
        Meta::Path(path) => {
          let ident = path.require_ident()?.to_string();

          match ident.as_str() {
            "no_prefix" => no_prefix = true,
            _ => bail!(path, format!("Unknown attribute `{ident}`")),
          };
        }
      }
    }
  }

  let name = proto_name.unwrap_or_else(|| ccase!(pascal, enum_ident.to_string()));
  let full_name = full_name.unwrap_or_else(|| name.clone());

  let file = file.ok_or(error!(Span::call_site(), "Missing file attribute"))?;
  let package = package.ok_or(error!(Span::call_site(), "Missing package attribute"))?;

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
