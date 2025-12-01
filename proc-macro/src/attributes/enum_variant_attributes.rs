use crate::*;

pub struct EnumVariantAttrs {
  pub name: String,
  pub options: ProtoOptions,
}

pub fn process_derive_enum_variants_attrs(
  enum_name: &str,
  rust_variant_name: &Ident,
  attrs: &Vec<Attribute>,
  no_prefix: bool,
) -> Result<EnumVariantAttrs, Error> {
  let mut options: Option<TokenStream2> = None;
  let mut name: Option<String> = None;

  for attr in attrs {
    if !attr.path().is_ident("proto") {
      continue;
    }

    let args = attr.parse_args::<PunctuatedParser<Meta>>().unwrap();

    for meta in args.inner {
      match meta {
        Meta::NameValue(nameval) => {
          if nameval.path.is_ident("options") {
            let func_call = nameval.value;

            options = Some(quote! { #func_call });
          } else if nameval.path.is_ident("name") {
            name = Some(extract_string_lit(&nameval.value).unwrap());
          }
        }
        Meta::List(list) => {
          if list.path.is_ident("options") {
            let exprs = list.parse_args::<PunctuatedParser<Expr>>().unwrap().inner;

            options = Some(quote! { vec! [ #exprs ] });
          }
        }
        Meta::Path(_) => {}
      };
    }
  }

  let name = if let Some(name) = name {
    name
  } else {
    let plain_name = ccase!(constant, rust_variant_name.to_string());

    if no_prefix {
      plain_name
    } else {
      let prefix = ccase!(constant, enum_name);
      format!("{}_{}", prefix, plain_name)
    }
  };

  Ok(EnumVariantAttrs {
    options: attributes::ProtoOptions(options),
    name,
  })
}
