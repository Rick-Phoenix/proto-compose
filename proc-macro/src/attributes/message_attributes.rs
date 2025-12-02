use crate::*;

pub struct MessageAttrs {
  pub reserved_names: ReservedNames,
  pub reserved_numbers: ReservedNumbers,
  pub options: Vec<Expr>,
  pub name: String,
  pub full_name: String,
  pub file: String,
  pub package: String,
  pub nested_messages: Vec<Ident>,
  pub nested_enums: Vec<Ident>,
  pub direct: bool,
  pub from_proto: Option<PathOrClosure>,
  pub into_proto: Option<PathOrClosure>,
  pub shadow_derives: Option<MetaList>,
}

pub fn process_derive_message_attrs(
  rust_name: &Ident,
  attrs: &Vec<Attribute>,
) -> Result<MessageAttrs, Error> {
  let mut reserved_names = ReservedNames::default();
  let mut reserved_numbers = ReservedNumbers::default();
  let mut options: Vec<Expr> = Vec::new();
  let mut proto_name: Option<String> = None;
  let mut full_name: Option<String> = None;
  let mut file: Option<String> = None;
  let mut package: Option<String> = None;
  let mut direct = false;
  let mut nested_messages: Vec<Ident> = Vec::new();
  let mut nested_enums: Vec<Ident> = Vec::new();
  let mut from_proto: Option<PathOrClosure> = None;
  let mut into_proto: Option<PathOrClosure> = None;
  let mut shadow_derives: Option<MetaList> = None;

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
            "nested_messages" => {
              let idents = list.parse_args::<PunctuatedParser<Ident>>()?.inner;

              nested_messages.extend(idents.into_iter());
            }
            "nested_enums" => {
              let idents = list.parse_args::<PunctuatedParser<Ident>>()?.inner;

              nested_enums.extend(idents.into_iter());
            }
            "derive" => shadow_derives = Some(list),
            _ => {}
          };
        }
        Meta::NameValue(nv) => {
          let ident = get_ident_or_continue!(nv.path);

          match ident.as_str() {
            "from_proto" => {
              let expr = parse_path_or_closure(nv.value)?;

              from_proto = Some(expr);
            }
            "into_proto" => {
              let expr = parse_path_or_closure(nv.value)?;

              into_proto = Some(expr);
            }
            "name" => {
              proto_name = Some(extract_string_lit(&nv.value)?);
            }
            "full_name" => {
              full_name = Some(extract_string_lit(&nv.value)?);
            }
            "reserved_names" => {
              reserved_names = ReservedNames::Expr(nv.value);
            }
            "file" => {
              file = Some(extract_string_lit(&nv.value)?);
            }
            "package" => {
              package = Some(extract_string_lit(&nv.value)?);
            }
            _ => {}
          };
        }
        Meta::Path(path) => {
          if path.is_ident("direct") {
            direct = true;
          }
        }
      }
    }
  }

  let file = file.ok_or(error!(Span::call_site(), "File attribute is missing"))?;
  let package = package.ok_or(error!(Span::call_site(), "Package attribute is missing"))?;

  let name = proto_name.unwrap_or_else(|| ccase!(pascal, rust_name.to_string()));

  Ok(MessageAttrs {
    reserved_names,
    reserved_numbers,
    options,
    full_name: full_name.unwrap_or_else(|| name.clone()),
    name,
    file,
    package,
    nested_messages,
    nested_enums,
    direct,
    from_proto,
    into_proto,
    shadow_derives,
  })
}
