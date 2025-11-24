use crate::*;

pub struct ModuleEnumAttrs {
  pub reserved_numbers: ReservedNumbers,
  pub name: String,
}

pub fn process_module_enum_attrs(
  rust_name: &Ident,
  attrs: &Vec<Attribute>,
) -> Result<ModuleEnumAttrs, Error> {
  let mut reserved_numbers = ReservedNumbers::default();
  let mut proto_name: Option<String> = None;

  for attr in attrs {
    if !attr.path().is_ident("proto") {
      continue;
    }

    let args = attr.parse_args::<PunctuatedParser<Meta>>().unwrap();

    for arg in args.inner {
      match arg {
        Meta::List(list) => {
          if list.path.is_ident("reserved_numbers") {
            let numbers = list.parse_args::<ReservedNumbers>().unwrap();

            reserved_numbers = numbers;
          }
        }
        Meta::NameValue(nameval) => {
          if nameval.path.is_ident("name") {
            proto_name = Some(extract_string_lit(&nameval.value).unwrap());
          }
        }
        Meta::Path(_) => {}
      }
    }
  }

  let name = proto_name.unwrap_or_else(|| ccase!(pascal, rust_name.to_string()));

  Ok(ModuleEnumAttrs {
    reserved_numbers,
    name,
  })
}
