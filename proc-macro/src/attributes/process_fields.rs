use convert_case::ccase;
use syn::ExprCall;

use crate::*;

pub struct FieldAttrs2 {
  pub tag: i32,
  pub validator: Option<ValidatorExpr>,
  pub options: ProtoOptions,
  pub name: String,
  pub kind: ProtoFieldType,
  pub custom_type: Option<Path>,
  pub oneof_tags: Vec<i32>,
}

pub fn process_derive_field(field: &mut Field) -> Result<Option<FieldAttrs>, Error> {
  let Field {
    attrs, ident, ty, ..
  } = field;

  let field_ident = ident.as_mut().unwrap();

  let mut validator: Option<ValidatorExpr> = None;
  let mut tag: Option<i32> = None;
  let mut options: Option<TokenStream2> = None;
  let mut name: Option<String> = None;
  let mut custom_type: Option<Path> = None;
  let mut kind = ProtoFieldType::default();
  let mut is_ignored = false;
  let mut oneof_tags: Vec<i32> = Vec::new();
  let mut proto_type: Option<ProtoType> = None;

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
          } else if list.path.is_ident("oneof_tags") {
            let nums = list.parse_args::<NumList>()?.list;

            oneof_tags = nums;
          }
        }
        Meta::Path(path) => {
          if path.is_ident("ignore") {
            is_ignored = true;
          } else if path.is_ident("oneof") {
            kind = ProtoFieldType::Oneof;
          } else if path.is_ident("enum_") {
            kind = ProtoFieldType::Enum;
          } else if path.is_ident("message") {
            kind = ProtoFieldType::Message;
          }
        }
      };
    }
  }

  let tag = if is_ignored || !kind.is_normal() {
    0
  } else {
    tag.ok_or(spanned_error!(&field_ident, "Field tag is missing"))?
  };

  if !is_ignored {
    Ok(Some(FieldAttrs {
      validator,
      tag,
      options: attributes::ProtoOptions(options),
      name: name.unwrap_or_else(|| ccase!(snake, field_ident.to_string())),
      custom_type,
      kind,
      oneof_tags,
    }))
  } else {
    Ok(None)
  }
}
