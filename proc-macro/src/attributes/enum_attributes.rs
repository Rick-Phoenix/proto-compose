use crate::*;

pub struct EnumAttrs {
  pub reserved_names: ReservedNames,
  pub reserved_numbers: ReservedNumbers,
  pub options: Vec<Expr>,
  pub name: String,
  pub file: String,
  pub package: String,
  pub full_name: String,
  pub no_prefix: bool,
}

pub fn process_derive_enum_attrs(
  rust_name: &Ident,
  attrs: &Vec<Attribute>,
) -> Result<EnumAttrs, Error> {
  let mut reserved_names = ReservedNames::default();
  let mut reserved_numbers = ReservedNumbers::default();
  let mut options: Vec<Expr> = Vec::new();
  let mut proto_name: Option<String> = None;
  let mut full_name: Option<String> = None;
  let mut file: Option<String> = None;
  let mut package: Option<String> = None;
  let mut no_prefix = false;

  for attr in attrs {
    if !attr.path().is_ident("proto") {
      continue;
    }

    let args = attr.parse_args::<PunctuatedParser<Meta>>()?;

    for arg in args.inner {
      match arg {
        Meta::List(list) => {
          let ident = get_ident_or_continue!(list.path);

          match ident.as_str() {
            "reserved_names" => {
              let names = list.parse_args::<StringList>()?;

              reserved_names = ReservedNames::List(names.list);
            }
            "reserved_numbers" => {
              let numbers = list.parse_args::<ReservedNumbers>()?;

              reserved_numbers = numbers;
            }
            "options" => {
              let exprs = list.parse_args::<PunctuatedParser<Expr>>()?.inner;

              options = exprs.into_iter().collect();
            }
            _ => {}
          };
        }
        Meta::NameValue(nv) => {
          let ident = get_ident_or_continue!(nv.path);

          match ident.as_str() {
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
            _ => {}
          };
        }
        Meta::Path(path) => {
          let ident = get_ident_or_continue!(path);

          match ident.as_str() {
            "no_prefix" => no_prefix = true,
            _ => {}
          };
        }
      }
    }
  }

  let name = proto_name.unwrap_or_else(|| ccase!(pascal, rust_name.to_string()));
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
  })
}
