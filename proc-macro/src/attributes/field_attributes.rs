use convert_case::ccase;
use syn::ExprCall;

use crate::*;

#[derive(Default, Debug, Clone)]
pub enum ProtoFieldType {
  Message(Option<Path>),
  Enum(Option<Path>),
  Oneof,
  Map(ProtoMap),
  Sint32,
  #[default]
  None,
}

impl ProtoFieldType {
  pub fn is_message(&self) -> bool {
    matches!(self, Self::Message(_))
  }

  pub fn is_enum(&self) -> bool {
    matches!(self, Self::Enum(_))
  }

  pub fn is_oneof(&self) -> bool {
    matches!(self, Self::Oneof)
  }

  pub fn is_none(&self) -> bool {
    matches!(self, Self::None)
  }
}

pub struct FieldAttrs {
  pub tag: i32,
  pub validator: Option<ValidatorExpr>,
  pub options: ProtoOptions,
  pub name: String,
  pub kind: ProtoFieldType,
  pub custom_type: Option<Path>,
  pub oneof_tags: Vec<i32>,
  pub proto_type: Option<Path>,
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
  let mut kind = ProtoFieldType::default();
  let mut is_ignored = false;
  let mut proto_type: Option<Path> = None;
  let mut oneof_tags: Vec<i32> = Vec::new();

  for attr in attrs {
    if !attr.path().is_ident("proto") {
      continue;
    }

    let args = attr.parse_args::<PunctuatedParser<Meta>>()?;

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
          }
        }
        Meta::List(list) => {
          let ident = if let Some(ident) = list.path.get_ident() {
            ident.to_string()
          } else {
            continue;
          };

          match ident.as_str() {
            "message" => {
              let message_path = list.parse_args::<Path>()?;

              kind = ProtoFieldType::Message(Some(message_path));
            }
            "enum_" => {
              let enum_path = list.parse_args::<Path>()?;

              kind = ProtoFieldType::Enum(Some(enum_path));
            }
            "options" => {
              let exprs = list.parse_args::<PunctuatedParser<Expr>>().unwrap().inner;

              options = Some(quote! { vec! [ #exprs ] });
            }
            "map" => {
              let map_data = list.parse_args::<ProtoMap>()?;

              kind = ProtoFieldType::Map(map_data);
            }
            "type_" => {
              custom_type = Some(list.parse_args::<Path>()?);
            }
            "proto_type" => {
              proto_type = Some(list.parse_args::<Path>()?);
            }
            _ => {}
          };
        }
        Meta::Path(path) => {
          let ident = if let Some(ident) = path.get_ident() {
            ident.to_string()
          } else {
            continue;
          };

          match ident.as_str() {
            "ignore" => is_ignored = true,
            "oneof" => kind = ProtoFieldType::Oneof,
            "enum_" => kind = ProtoFieldType::Enum(None),
            "message" => kind = ProtoFieldType::Message(None),
            "sint32" => kind = ProtoFieldType::Sint32,

            _ => {}
          };
        }
      };
    }
  }

  let tag = if let Some(tag) = tag {
    tag
  } else if is_ignored || kind.is_oneof() {
    0
  } else {
    return Err(spanned_error!(original_name, "Field tag is missing"));
  };

  if !is_ignored {
    Ok(Some(FieldAttrs {
      validator,
      tag,
      options: attributes::ProtoOptions(options),
      name: name.unwrap_or_else(|| ccase!(snake, original_name.to_string())),
      custom_type,
      kind,
      oneof_tags,
      proto_type,
    }))
  } else {
    Ok(None)
  }
}
