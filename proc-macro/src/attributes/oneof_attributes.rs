use crate::*;

pub struct OneofAttrs {
  pub options: Option<Expr>,
  pub name: String,
  pub required: bool,
  pub direct: bool,
  pub from_proto: Option<PathOrClosure>,
  pub into_proto: Option<PathOrClosure>,
  pub shadow_derives: Option<MetaList>,
  pub backend: Backend,
}

pub fn process_oneof_attrs(enum_ident: &Ident, attrs: &[Attribute]) -> Result<OneofAttrs, Error> {
  let mut options: Option<Expr> = None;
  let mut name: Option<String> = None;
  let mut required = false;
  let mut direct = false;
  let mut from_proto: Option<PathOrClosure> = None;
  let mut into_proto: Option<PathOrClosure> = None;
  let mut shadow_derives: Option<MetaList> = None;
  let mut backend = Backend::default();

  for arg in filter_attributes(attrs, &["proto"])? {
    match arg {
      Meta::Path(path) => {
        let ident = path.require_ident()?.to_string();

        match ident.as_str() {
          "required" => required = true,
          "direct" => direct = true,
          _ => bail!(path, "Unknown attribute `{ident}`"),
        };
      }
      Meta::List(list) => {
        let ident = list.path.require_ident()?.to_string();

        match ident.as_str() {
          "derive" => shadow_derives = Some(list),
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
          "from_proto" => {
            from_proto = Some(nv.value.as_path_or_closure()?);
          }
          "into_proto" => {
            into_proto = Some(nv.value.as_path_or_closure()?);
          }
          "name" => name = Some(nv.value.as_string()?),
          _ => bail!(nv.path, "Unknown attribute `{ident}`"),
        };
      }
    }
  }

  Ok(OneofAttrs {
    options,
    name: name.unwrap_or_else(|| ccase!(snake, enum_ident.to_string())),
    required,
    direct,
    from_proto,
    into_proto,
    shadow_derives,
    backend,
  })
}
