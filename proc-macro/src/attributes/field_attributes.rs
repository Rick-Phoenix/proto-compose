use convert_case::ccase;
use syn::ExprCall;

use crate::*;

pub struct FieldAttrs {
  pub tag: i32,
  pub validator: Option<ValidatorExpr>,
  pub options: ProtoOptions,
  pub name: String,
  pub is_oneof: bool,
  pub custom_type: Option<Path>,
}

pub enum ValidatorExpr {
  Closure(ExprClosure),
  Call(ExprCall),
}

pub fn process_derive_field_attrs(
  original_name: &Ident,
  attrs: &Vec<Attribute>,
) -> Result<Option<FieldAttrs>, Error> {
  let mut validator: Option<ValidatorExpr> = None;
  let mut tag: Option<i32> = None;
  let mut options: Option<TokenStream2> = None;
  let mut name: Option<String> = None;
  let mut custom_type: Option<Path> = None;
  let mut is_ignored = false;
  let mut is_oneof = false;

  for attr in attrs {
    if !attr.path().is_ident("proto") {
      continue;
    }

    let args = attr.parse_args::<PunctuatedParser<Meta>>().unwrap();

    for meta in args.inner {
      match meta {
        Meta::NameValue(nameval) => {
          if nameval.path.is_ident("validate") {
            if let Expr::Closure(closure) = nameval.value {
              validator = Some(ValidatorExpr::Closure(closure));
            } else if let Expr::Call(call) = nameval.value {
              validator = Some(ValidatorExpr::Call(call));
            } else {
              panic!("Invalid");
            }
          } else if nameval.path.is_ident("tag") {
            tag = Some(extract_i32(&nameval.value).unwrap());
          } else if nameval.path.is_ident("options") {
            let func_call = nameval.value;

            options = Some(quote! { #func_call });
          } else if nameval.path.is_ident("name") {
            name = Some(extract_string_lit(&nameval.value).unwrap());
          } else if nameval.path.is_ident("type_") {
            custom_type = Some(extract_path(nameval.value)?);
          }
        }
        Meta::List(list) => {
          if list.path.is_ident("options") {
            let exprs = list.parse_args::<PunctuatedParser<Expr>>().unwrap().inner;

            options = Some(quote! { vec! [ #exprs ] });
          }
        }
        Meta::Path(path) => {
          if path.is_ident("ignore") {
            is_ignored = true;
          } else if path.is_ident("oneof") {
            is_oneof = true;
          }
        }
      };
    }
  }

  let tag = if is_ignored || is_oneof {
    0
  } else {
    tag.ok_or(spanned_error!(original_name, "Field tag is missing"))?
  };

  if !is_ignored {
    Ok(Some(FieldAttrs {
      validator,
      tag,
      options: attributes::ProtoOptions(options),
      name: name.unwrap_or_else(|| ccase!(snake, original_name.to_string())),
      is_oneof,
      custom_type,
    }))
  } else {
    Ok(None)
  }
}
