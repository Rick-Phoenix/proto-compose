use crate::*;

pub struct OneofAttrs {
  pub options: Option<Expr>,
  pub name: String,
  pub from_proto: Option<PathOrClosure>,
  pub into_proto: Option<PathOrClosure>,
  pub shadow_derives: Option<MetaList>,
}

pub fn process_oneof_attrs(enum_ident: &Ident, attrs: &[Attribute]) -> Result<OneofAttrs, Error> {
  let mut options: Option<Expr> = None;
  let mut name: Option<String> = None;
  let mut from_proto: Option<PathOrClosure> = None;
  let mut into_proto: Option<PathOrClosure> = None;
  let mut shadow_derives: Option<MetaList> = None;

  parse_filtered_attrs(attrs, &["proto"], |meta| {
    let ident = meta.path.require_ident()?.to_string();

    match ident.as_str() {
      "derive" => {
        let list = meta.parse_list::<MetaList>()?;

        shadow_derives = Some(list);
      }
      "options" => {
        options = Some(meta.expr_value()?);
      }
      "from_proto" => {
        from_proto = Some(meta.expr_value()?.as_path_or_closure()?);
      }
      "into_proto" => {
        into_proto = Some(meta.expr_value()?.as_path_or_closure()?);
      }
      "name" => name = Some(meta.expr_value()?.as_string()?),
      _ => return Err(meta.error("Unknown attribute")),
    };

    Ok(())
  })?;

  Ok(OneofAttrs {
    options,
    name: name.unwrap_or_else(|| ccase!(snake, enum_ident.to_string())),
    from_proto,
    into_proto,
    shadow_derives,
  })
}
