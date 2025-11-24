use crate::*;

pub struct EnumAttrs {
  pub reserved_names: ReservedNames,
  pub reserved_numbers: ReservedNumbers,
  pub options: ProtoOptions,
  pub name: String,
  pub file: String,
  pub package: String,
  pub full_name: String,
}

pub fn process_derive_enum_attrs(
  rust_name: &Ident,
  attrs: &Vec<Attribute>,
) -> Result<EnumAttrs, Error> {
  let mut reserved_names = ReservedNames::default();
  let mut reserved_numbers = ReservedNumbers::default();
  let mut options: Option<TokenStream2> = None;
  let mut proto_name: Option<String> = None;
  let mut full_name: Option<String> = None;
  let mut file: Option<String> = None;
  let mut package: Option<String> = None;

  for attr in attrs {
    if !attr.path().is_ident("proto") {
      continue;
    }

    let args = attr.parse_args::<PunctuatedParser<Meta>>().unwrap();

    for arg in args.inner {
      match arg {
        Meta::List(list) => {
          if list.path.is_ident("reserved_names") {
            let names = list.parse_args::<StringList>().unwrap();

            reserved_names = ReservedNames::List(names.list);
          } else if list.path.is_ident("reserved_numbers") {
            let numbers = list.parse_args::<ReservedNumbers>().unwrap();

            reserved_numbers = numbers;
          } else if list.path.is_ident("options") {
            let exprs = list.parse_args::<PunctuatedParser<Expr>>().unwrap().inner;

            options = Some(quote! { vec! [ #exprs ] });
          }
        }
        Meta::NameValue(nameval) => {
          if nameval.path.is_ident("options") {
            let func_call = nameval.value;

            options = Some(quote! { #func_call });
          } else if nameval.path.is_ident("name") {
            proto_name = Some(extract_string_lit(&nameval.value).unwrap());
          } else if nameval.path.is_ident("reserved_names") {
            reserved_names = ReservedNames::Expr(nameval.value);
          } else if nameval.path.is_ident("full_name") {
            full_name = Some(extract_string_lit(&nameval.value).unwrap());
          } else if nameval.path.is_ident("package") {
            package = Some(extract_string_lit(&nameval.value).unwrap());
          } else if nameval.path.is_ident("file") {
            file = Some(extract_string_lit(&nameval.value).unwrap());
          }
        }
        Meta::Path(_) => {}
      }
    }
  }

  let name = proto_name.unwrap_or_else(|| ccase!(pascal, rust_name.to_string()));
  let full_name = full_name.unwrap_or_else(|| name.clone());
  let file = file.expect("missing file");
  let package = package.expect("missing package");

  Ok(EnumAttrs {
    reserved_names,
    reserved_numbers,
    options: attributes::ProtoOptions(options),
    name,
    file,
    package,
    full_name,
  })
}
