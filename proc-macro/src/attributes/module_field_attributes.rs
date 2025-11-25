use convert_case::ccase;

use crate::*;

pub struct ModuleFieldAttrs {
  pub tag: Option<i32>,
  pub name: String,
  pub kind: ProtoFieldType,
  pub custom_type: Option<Path>,
}

pub fn process_module_field_attrs(
  original_name: &Ident,
  attrs: &Vec<Attribute>,
) -> Result<Option<ModuleFieldAttrs>, Error> {
  let mut tag: Option<i32> = None;
  let mut name: Option<String> = None;
  let mut custom_type: Option<Path> = None;
  let mut kind = ProtoFieldType::default();

  for attr in attrs {
    if !attr.path().is_ident("proto") {
      continue;
    }

    let args = attr.parse_args::<PunctuatedParser<Meta>>().unwrap();

    for meta in args.inner {
      match meta {
        Meta::NameValue(nameval) => {
          if nameval.path.is_ident("tag") {
            tag = Some(extract_i32(&nameval.value).unwrap());
          } else if nameval.path.is_ident("name") {
            name = Some(extract_string_lit(&nameval.value).unwrap());
          } else if nameval.path.is_ident("type_") {
            custom_type = Some(extract_path(nameval.value)?);
          }
        }
        Meta::Path(path) => {
          if path.is_ident("ignore") {
            return Ok(None);
          } else if path.is_ident("oneof") {
            kind = ProtoFieldType::Oneof;
          } else if path.is_ident("enum_") {
            kind = ProtoFieldType::Enum;
          } else if path.is_ident("message") {
            kind = ProtoFieldType::Message;
          }
        }
        Meta::List(_) => {}
      };
    }
  }

  Ok(Some(ModuleFieldAttrs {
    tag,
    name: name.unwrap_or_else(|| ccase!(snake, original_name.to_string())),
    custom_type,
    kind,
  }))
}
