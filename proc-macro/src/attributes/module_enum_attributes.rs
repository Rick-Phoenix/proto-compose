use crate::*;

pub struct ModuleEnumAttrs {
  pub name: String,
}

pub fn process_module_enum_attrs(
  rust_name: &Ident,
  attrs: &Vec<Attribute>,
) -> Result<ModuleEnumAttrs, Error> {
  let mut proto_name: Option<String> = None;

  for attr in attrs {
    if !attr.path().is_ident("proto") {
      continue;
    }

    let args = attr.parse_args::<PunctuatedParser<Meta>>()?;

    for arg in args.inner {
      match arg {
        Meta::List(_) => {}
        Meta::NameValue(nv) => {
          let ident = get_ident_or_continue!(nv.path);

          match ident.as_str() {
            "name" => {
              proto_name = Some(extract_string_lit(&nv.value)?);
            }
            _ => {}
          };
        }
        Meta::Path(_) => {}
      }
    }
  }

  let name = proto_name.unwrap_or_else(|| ccase!(pascal, rust_name.to_string()));

  Ok(ModuleEnumAttrs { name })
}
