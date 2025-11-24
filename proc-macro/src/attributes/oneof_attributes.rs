use crate::*;

#[derive(Debug)]
pub struct OneofAttrs {
  pub options: ProtoOptions,
  pub name: String,
  pub required: bool,
}

pub fn process_oneof_attrs(
  enum_name: &Ident,
  attrs: &Vec<Attribute>,
  is_in_module_macro: bool,
) -> OneofAttrs {
  let mut options: Option<TokenStream2> = None;
  let mut name: Option<String> = None;
  let mut required = false;

  for attr in attrs {
    if !attr.path().is_ident("proto") {
      continue;
    }

    let args = attr.parse_args::<PunctuatedParser<Meta>>().unwrap();

    for arg in args.inner {
      match arg {
        Meta::Path(path) => {
          if path.is_ident("required") {
            required = true;
          }
        }
        Meta::List(list) => {
          if !is_in_module_macro && list.path.is_ident("options") {
            let exprs = list.parse_args::<PunctuatedParser<Expr>>().unwrap().inner;

            options = Some(quote! { vec! [ #exprs ] });
          }
        }
        Meta::NameValue(nameval) => {
          if !is_in_module_macro && nameval.path.is_ident("options") {
            let func_call = nameval.value;

            options = Some(quote! { #func_call });
          } else if nameval.path.is_ident("name") {
            name = Some(extract_string_lit(&nameval.value).unwrap());
          }
        }
      }
    }
  }

  OneofAttrs {
    options: attributes::ProtoOptions(options),
    name: name.unwrap_or_else(|| ccase!(snake, enum_name.to_string())),
    required,
  }
}
