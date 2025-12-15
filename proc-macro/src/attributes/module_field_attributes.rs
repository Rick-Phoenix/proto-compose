use convert_case::ccase;

use crate::*;

pub struct ModuleFieldAttrs {
  pub tag: Option<i32>,
  pub name: String,
  pub oneof_info: Option<OneofInfo>,
  pub is_ignored: bool,
}

pub fn process_module_field_attrs(
  original_name: &Ident,
  attrs: &[Attribute],
) -> Result<ModuleFieldAttrs, Error> {
  let mut tag: Option<i32> = None;
  let mut name: Option<String> = None;
  let mut oneof_info: Option<OneofInfo> = None;
  let mut is_ignored = false;

  for arg in filter_attributes(attrs, &["proto"])? {
    match arg {
      Meta::NameValue(nv) => {
        let ident = get_ident_or_continue!(nv.path);

        match ident.as_str() {
          "tag" => tag = Some(nv.value.as_int::<i32>()?),
          "name" => name = Some(nv.value.as_string()?),
          _ => {}
        };
      }
      Meta::Path(path) => {
        let ident = get_ident_or_continue!(path);

        match ident.as_str() {
          "ignore" => {
            is_ignored = true;
          }
          "oneof" => {
            oneof_info = Some(OneofInfo::default());
          }
          _ => {}
        };
      }
      Meta::List(list) => {
        let ident = get_ident_or_continue!(list.path);

        match ident.as_str() {
          "oneof" => {
            oneof_info = Some(list.parse_args::<OneofInfo>()?);
          }
          _ => {}
        };
      }
    };
  }

  Ok(ModuleFieldAttrs {
    tag,
    is_ignored,
    name: name.unwrap_or_else(|| ccase!(snake, original_name.to_string())),
    oneof_info,
  })
}
