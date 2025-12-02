use crate::*;

pub struct OneofAttrs {
  pub options: Vec<Expr>,
  pub name: String,
  pub required: bool,
  pub direct: bool,
  pub from_proto: Option<PathOrClosure>,
  pub into_proto: Option<PathOrClosure>,
  pub shadow_derives: Option<MetaList>,
}

pub fn process_oneof_attrs(enum_name: &Ident, attrs: &Vec<Attribute>) -> Result<OneofAttrs, Error> {
  let mut options: Vec<Expr> = Vec::new();
  let mut name: Option<String> = None;
  let mut required = false;
  let mut direct = false;
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
        Meta::Path(path) => {
          let ident = if let Some(ident) = path.get_ident() {
            ident.to_string()
          } else {
            continue;
          };

          match ident.as_str() {
            "required" => required = true,
            "direct" => direct = true,
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
            "name" => name = Some(extract_string_lit(&nv.value)?),
            _ => {}
          };
        }
      }
    }
  }

  Ok(OneofAttrs {
    options,
    name: name.unwrap_or_else(|| ccase!(snake, enum_name.to_string())),
    required,
    direct,
    from_proto,
    into_proto,
    shadow_derives,
  })
}
