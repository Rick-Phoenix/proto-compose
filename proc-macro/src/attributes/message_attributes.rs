use crate::*;

pub struct MessageAttrs {
  pub reserved_names: Vec<String>,
  pub reserved_numbers: ReservedNumbers,
  pub options: Option<Expr>,
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
  pub cel_rules: Option<Vec<Path>>,
  pub backend: Backend,
}

pub fn process_derive_message_attrs(
  rust_name: &Ident,
  attrs: &[Attribute],
) -> Result<MessageAttrs, Error> {
  let mut reserved_names: Vec<String> = Vec::new();
  let mut reserved_numbers = ReservedNumbers::default();
  let mut options: Option<Expr> = None;
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
  let mut cel_rules: Option<Vec<Path>> = None;
  let mut backend = Backend::default();

  for arg in filter_attributes(attrs, &["proto"])? {
    match arg {
      Meta::List(list) => {
        let ident = list.path.require_ident()?.to_string();

        match ident.as_str() {
          "cel_rules" => {
            cel_rules = Some(list.parse_args::<PathList>()?.list);
          }
          "reserved_names" => {
            let names = list.parse_args::<StringList>()?;

            reserved_names = names.list;
          }
          "reserved_numbers" => {
            let numbers = list.parse_args::<ReservedNumbers>()?;

            reserved_numbers = numbers;
          }
          "nested_messages" => {
            let idents = list.parse_args::<IdentList>()?.list;

            nested_messages.extend(idents.into_iter());
          }
          "nested_enums" => {
            let idents = list.parse_args::<IdentList>()?.list;

            nested_enums.extend(idents.into_iter());
          }
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
          "name" => {
            proto_name = Some(nv.value.as_string()?);
          }
          "full_name" => {
            full_name = Some(nv.value.as_string()?);
          }
          "file" => {
            file = Some(nv.value.as_string()?);
          }
          "package" => {
            package = Some(nv.value.as_string()?);
          }
          _ => bail!(nv.path, "Unknown attribute `{ident}`"),
        };
      }
      Meta::Path(path) => {
        let ident = path.require_ident()?.to_string();

        match ident.as_str() {
          "direct" => direct = true,
          _ => bail!(path, "Unknown attribute `{ident}`"),
        };
      }
    }
  }

  let file = file.ok_or(error_call_site!(
    r#"`file` attribute is missing. Use the `proto_module` macro on the surrounding module or set it manually with #[proto(file = "my_file.proto")]"#
  ))?;
  let package = package.ok_or(error_call_site!(r#"`package` attribute is missing. Use the `proto_module` macro on the surrounding module or set it manually with #[proto(package = "mypackage.v1")]"#))?;

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
    cel_rules,
    backend,
  })
}
