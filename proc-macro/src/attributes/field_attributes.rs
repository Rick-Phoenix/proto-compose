use convert_case::ccase;
use syn::ExprCall;

use crate::*;

#[derive(Default, Debug, Clone)]
pub enum ProtoFieldKind {
  Message(MessageInfo),
  Enum(Option<Path>),
  Oneof(OneofInfo),
  Map(ProtoMap),
  Sint32,
  #[default]
  None,
}

#[derive(Default, Debug, Clone)]
pub enum ItemPath {
  Path(Path),
  Suffixed,
  #[default]
  None,
}

impl ItemPath {
  pub fn is_suffixed(&self) -> bool {
    matches!(self, Self::Suffixed)
  }

  pub fn is_none(&self) -> bool {
    matches!(self, Self::None)
  }
}

impl ToTokens for ItemPath {
  fn to_tokens(&self, tokens: &mut TokenStream2) {
    match self {
      Self::Path(path) => tokens.extend(path.to_token_stream()),
      _ => {}
    };
  }
}

impl ProtoFieldKind {
  pub fn is_message(&self) -> bool {
    matches!(self, Self::Message(_))
  }

  pub fn is_enum(&self) -> bool {
    matches!(self, Self::Enum(_))
  }

  pub fn is_oneof(&self) -> bool {
    matches!(self, Self::Oneof { .. })
  }

  pub fn is_none(&self) -> bool {
    matches!(self, Self::None)
  }
}

#[derive(Clone)]
pub struct FieldAttrs {
  pub tag: i32,
  pub validator: Option<ValidatorExpr>,
  pub options: ProtoOptions,
  pub name: String,
  pub kind: ProtoFieldKind,
  pub from_proto: Option<PathOrClosure>,
  pub into_proto: Option<PathOrClosure>,
  pub is_ignored: bool,
}

#[derive(Clone)]
pub enum ValidatorExpr {
  Closure(ExprClosure),
  Call(ExprCall),
}

pub fn process_derive_field_attrs(
  original_name: &Ident,
  attrs: &Vec<Attribute>,
) -> Result<FieldAttrs, Error> {
  let mut validator: Option<ValidatorExpr> = None;
  let mut tag: Option<i32> = None;
  let mut options: Option<TokenStream2> = None;
  let mut name: Option<String> = None;
  let mut kind = ProtoFieldKind::default();
  let mut is_ignored = false;
  let mut from_proto: Option<PathOrClosure> = None;
  let mut into_proto: Option<PathOrClosure> = None;
  let mut oneof_info: Option<OneofInfo> = None;

  for attr in attrs {
    if !attr.path().is_ident("proto") {
      continue;
    }

    let args = attr.parse_args::<PunctuatedParser<Meta>>()?;

    for meta in args.inner {
      match meta {
        Meta::NameValue(nv) => {
          let ident = get_ident_or_continue!(nv.path);

          match ident.as_str() {
            "validate" => {
              validator = match nv.value {
                Expr::Closure(closure) => Some(ValidatorExpr::Closure(closure)),
                Expr::Call(call) => Some(ValidatorExpr::Call(call)),
                _ => panic!("Invalid validator"),
              };
            }
            "from_proto" => {
              let value = parse_path_or_closure(nv.value)?;

              from_proto = Some(value);
            }
            "into_proto" => {
              let value = parse_path_or_closure(nv.value)?;

              into_proto = Some(value);
            }
            "tag" => {
              tag = Some(extract_i32(&nv.value).unwrap());
            }
            "options" => {
              let func_call = nv.value;

              options = Some(quote! { #func_call });
            }
            "name" => {
              name = Some(extract_string_lit(&nv.value).unwrap());
            }
            _ => {}
          };
        }
        Meta::List(list) => {
          let ident = get_ident_or_continue!(list.path);

          match ident.as_str() {
            "oneof" => {
              let mut info = list.parse_args::<OneofInfo>()?;

              if let Some(previous) = &mut oneof_info {
                let previous = std::mem::take(previous);

                if previous.default {
                  info.default = true;
                }

                if !previous.path.is_none() {
                  info.path = previous.path;
                }

                info.tags.extend(previous.tags);
              } else {
                oneof_info = Some(info.clone());
              }

              kind = ProtoFieldKind::Oneof(info);
            }
            "message" => {
              let message_info = list.parse_args::<MessageInfo>()?;

              kind = ProtoFieldKind::Message(message_info);
            }
            "enum_" => {
              let enum_path = list.parse_args::<Path>()?;

              kind = ProtoFieldKind::Enum(Some(enum_path));
            }
            "options" => {
              let exprs = list.parse_args::<PunctuatedParser<Expr>>().unwrap().inner;

              options = Some(quote! { vec! [ #exprs ] });
            }
            "map" => {
              let map_data = list.parse_args::<ProtoMap>()?;

              kind = ProtoFieldKind::Map(map_data);
            }

            _ => {}
          };
        }
        Meta::Path(path) => {
          let ident = get_ident_or_continue!(path);

          match ident.as_str() {
            "ignore" => is_ignored = true,
            "oneof" => kind = ProtoFieldKind::Oneof(OneofInfo::default()),
            "enum_" => kind = ProtoFieldKind::Enum(None),
            "message" => kind = ProtoFieldKind::Message(MessageInfo::default()),
            "sint32" => kind = ProtoFieldKind::Sint32,

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

  Ok(FieldAttrs {
    validator,
    tag,
    options: attributes::ProtoOptions(options),
    name: name.unwrap_or_else(|| ccase!(snake, original_name.to_string())),
    kind,
    from_proto,
    into_proto,
    is_ignored,
  })
}
