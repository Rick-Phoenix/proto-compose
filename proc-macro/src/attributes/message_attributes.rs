use crate::*;

pub struct MessageAttrs {
  pub reserved_names: Vec<String>,
  pub reserved_numbers: ReservedNumbers,
  pub options: Option<Expr>,
  pub name: String,
  pub parent_message: Option<Ident>,
  pub from_proto: Option<PathOrClosure>,
  pub into_proto: Option<PathOrClosure>,
  pub shadow_derives: Option<MetaList>,
  pub cel_rules: Option<Vec<Path>>,
  pub is_direct: bool,
  pub no_auto_test: bool,
  pub extern_path: Option<String>,
}

pub fn process_derive_message_attrs(
  rust_name: &Ident,
  macro_attrs: MessageMacroAttrs,
  attrs: &[Attribute],
) -> Result<MessageAttrs, Error> {
  let mut reserved_names: Vec<String> = Vec::new();
  let mut reserved_numbers = ReservedNumbers::default();
  let mut options: Option<Expr> = None;
  let mut proto_name: Option<String> = None;
  let mut from_proto: Option<PathOrClosure> = None;
  let mut into_proto: Option<PathOrClosure> = None;
  let mut shadow_derives: Option<MetaList> = None;
  let mut cel_rules: Option<Vec<Path>> = None;
  let mut parent_message: Option<Ident> = None;

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
          "derive" => shadow_derives = Some(list),
          _ => bail!(list, "Unknown attribute `{ident}`"),
        };
      }
      Meta::NameValue(nv) => {
        let ident = nv.path.require_ident()?.to_string();

        match ident.as_str() {
          "parent_message" => {
            parent_message = Some(nv.value.as_path()?.require_ident()?.clone());
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
          _ => bail!(nv.path, "Unknown attribute `{ident}`"),
        };
      }
      Meta::Path(path) => {
        let ident = path.require_ident()?.to_string();

        match ident.as_str() {
          "direct" => bail!(
            path,
            "`direct` must be set as a proc macro argument, not as an attribute"
          ),
          _ => bail!(path, "Unknown attribute `{ident}`"),
        };
      }
    }
  }

  let name = proto_name.unwrap_or_else(|| ccase!(pascal, rust_name.to_string()));

  Ok(MessageAttrs {
    reserved_names,
    reserved_numbers,
    options,
    name,
    from_proto,
    into_proto,
    shadow_derives,
    cel_rules,
    is_direct: macro_attrs.is_direct,
    no_auto_test: macro_attrs.no_auto_test,
    extern_path: macro_attrs.extern_path,
    parent_message,
  })
}
