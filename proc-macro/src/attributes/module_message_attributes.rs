use crate::*;

pub struct ModuleMessageAttrs {
  pub reserved_names: ReservedNames,
  pub reserved_numbers: ReservedNumbers,
  pub name: String,
  pub nested_messages: Vec<Ident>,
  pub nested_enums: Vec<Ident>,
}

pub fn process_module_message_attrs(
  rust_name: &Ident,
  attrs: &Vec<Attribute>,
) -> Result<ModuleMessageAttrs, Error> {
  let mut reserved_names = ReservedNames::default();
  let mut reserved_numbers = ReservedNumbers::default();
  let mut proto_name: Option<String> = None;
  let mut nested_messages: Vec<Ident> = Vec::new();
  let mut nested_enums: Vec<Ident> = Vec::new();

  for attr in attrs {
    if !attr.path().is_ident("proto") {
      continue;
    }

    let args = attr.parse_args::<PunctuatedParser<Meta>>()?;

    for arg in args.inner {
      match arg {
        Meta::List(list) => {
          let ident = get_ident_or_continue!(list.path);

          match ident.as_str() {
            "reserved_names" => {
              let names = list.parse_args::<StringList>()?;

              reserved_names = ReservedNames::List(names.list);
            }
            "reserved_numbers" => {
              let numbers = list.parse_args::<ReservedNumbers>()?;

              reserved_numbers = numbers;
            }
            "nested_messages" => {
              let idents = list.parse_args::<PunctuatedParser<Ident>>()?.inner;

              nested_messages.extend(idents);
            }
            "nested_enums" => {
              let idents = list.parse_args::<PunctuatedParser<Ident>>()?.inner;

              nested_enums.extend(idents);
            }
            _ => {}
          };
        }
        Meta::NameValue(nv) => {
          let ident = get_ident_or_continue!(nv.path);

          match ident.as_str() {
            "name" => {
              proto_name = Some(extract_string_lit(&nv.value)?);
            }
            "reserved_names" => {
              reserved_names = ReservedNames::Expr(nv.value);
            }
            _ => {}
          };
        }
        Meta::Path(_) => {}
      }
    }
  }

  let name = proto_name.unwrap_or_else(|| ccase!(pascal, rust_name.to_string()));

  Ok(ModuleMessageAttrs {
    reserved_names,
    reserved_numbers,
    name,
    nested_messages,
    nested_enums,
  })
}
