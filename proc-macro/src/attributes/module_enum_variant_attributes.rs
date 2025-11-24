use crate::*;

pub struct ModuleEnumVariantAttrs {
  pub name: String,
  pub tag: Option<i32>,
}

pub fn process_module_enum_variants_attrs(
  enum_name: &str,
  rust_variant_name: &Ident,
  attrs: &Vec<Attribute>,
) -> Result<ModuleEnumVariantAttrs, Error> {
  let mut tag: Option<i32> = None;
  let mut name: Option<String> = None;

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
          }
        }
        Meta::List(_) => {}
        Meta::Path(_) => {}
      };
    }
  }

  let name = format!(
    "{}_{}",
    ccase!(constant, enum_name),
    name.unwrap_or_else(|| ccase!(constant, rust_variant_name.to_string()))
  );

  Ok(ModuleEnumVariantAttrs { tag, name })
}
