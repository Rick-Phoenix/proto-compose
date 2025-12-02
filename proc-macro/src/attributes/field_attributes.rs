use convert_case::ccase;
use syn::{
  parse::{ParseStream, Parser},
  spanned::Spanned,
  ExprCall,
};

use crate::*;

#[derive(Default, Debug, Clone)]
pub enum ItemPath {
  Path(Path),
  Proxied,
  #[default]
  None,
}

impl ItemPath {
  pub fn get_path_or_fallback(&self, fallback: Option<&Path>) -> Option<Path> {
    let output = if let Self::Path(path) = self {
      path.clone()
    } else if let Some(fallback) = fallback {
      let fallback = fallback.clone();

      if matches!(self, Self::Proxied) {
        append_proto_ident(fallback)
      } else {
        fallback
      }
    } else {
      return None;
    };

    Some(output)
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

// We probably should have an enum for this so that ignored fields don't hold the same state/info
#[derive(Clone)]
pub struct FieldAttrs {
  pub tag: i32,
  pub validator: Option<ValidatorExpr>,
  pub options: ProtoOptions,
  pub name: String,
  pub proto_field: ProtoField,
  pub from_proto: Option<PathOrClosure>,
  pub into_proto: Option<PathOrClosure>,
}

#[derive(Clone)]
pub enum FieldAttrData {
  Ignored { from_proto: Option<PathOrClosure> },
  Normal(FieldAttrs),
}

#[derive(Clone)]
pub enum ValidatorExpr {
  Closure(ExprClosure),
  Call(ExprCall),
}

pub fn process_derive_field_attrs(
  original_name: &Ident,
  rust_type: &RustType,
  attrs: &Vec<Attribute>,
) -> Result<FieldAttrData, Error> {
  let mut validator: Option<ValidatorExpr> = None;
  let mut tag: Option<i32> = None;
  let mut options: Option<TokenStream2> = None;
  let mut name: Option<String> = None;
  let mut proto_field: Option<ProtoField> = None;
  let mut is_ignored = false;
  let mut from_proto: Option<PathOrClosure> = None;
  let mut into_proto: Option<PathOrClosure> = None;

  let mut oneof_attrs: Vec<MetaList> = Vec::new();

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
            "options" => {
              let exprs = list.parse_args::<PunctuatedParser<Expr>>().unwrap().inner;

              options = Some(quote! { vec! [ #exprs ] });
            }

            "oneof" => {
              oneof_attrs.push(list);
            }

            "repeated" => {
              let args = list.parse_args::<Meta>()?;

              let fallback = if let RustType::Vec(path) = rust_type {
                path
              } else {
                panic!("This field was marked as repeated but it is not using Vec. Please set the output type manually");
              };

              let inner = ProtoType::from_meta(args, Some(fallback))?.unwrap();

              proto_field = Some(ProtoField::Repeated(inner));
            }

            "optional" => {
              let args = list.parse_args::<Meta>()?;

              let fallback = if let RustType::Option(path) = rust_type {
                path
              } else {
                panic!("Could not parse the option type");
              };

              let inner = ProtoType::from_meta(args, Some(fallback))?.unwrap();

              proto_field = Some(ProtoField::Optional(inner));
            }

            "map" => {
              let parser = |input: ParseStream| parse_map_with_context(input, rust_type);

              let map = parser.parse2(list.tokens)?;

              proto_field = Some(ProtoField::Map(map));
            }

            _ => {
              let fallback = rust_type.inner_path();

              if let Some(field_info) = ProtoType::from_meta_list(&ident, list, fallback)? {
                proto_field = Some(ProtoField::Single(field_info));
              }
            }
          };
        }
        Meta::Path(path) => {
          let ident = get_ident_or_continue!(path);

          match ident.as_str() {
            "ignore" => is_ignored = true,

            "oneof" => {}

            _ => {
              let fallback = rust_type.inner_path();

              let span = path.span();

              if let Some(parsed_kind) = ProtoType::from_ident(&ident, span, fallback)? {
                proto_field = Some(ProtoField::Single(parsed_kind));
              }
            }
          };
        }
      };
    }
  }

  if !oneof_attrs.is_empty() {
    let mut oneof_path = ItemPath::None;
    let mut oneof_tags: Vec<i32> = Vec::new();
    let mut use_default = false;

    let fallback = rust_type.inner_path();

    for attr in oneof_attrs {
      let OneofInfo {
        path,
        tags,
        default,
      } = attr.parse_args::<OneofInfo>()?;

      if !path.is_none() {
        oneof_path = path;
      }

      if !tags.is_empty() {
        oneof_tags = tags;
      }

      if default {
        use_default = true;
      }
    }

    let oneof_path = oneof_path.get_path_or_fallback(fallback).ok_or(error!(
      Span::call_site(),
      "Failed to infer the path to the oneof"
    ))?;

    proto_field = Some(ProtoField::Oneof {
      path: oneof_path,
      tags: oneof_tags,
      default: use_default,
    })
  }

  if is_ignored {
    return Ok(FieldAttrData::Ignored { from_proto });
  }

  let proto_field = if let Some(mut field) = proto_field {
    if let ProtoField::Single(proto_type) = &mut field && rust_type.is_option() {
      let inner = std::mem::take(proto_type);

      field = ProtoField::Optional(inner);
    }

    field
  } else {
    match rust_type {
      RustType::Map((k, v)) => {
        let keys = ProtoMapKeys::from_path(k)?;
        let values = ProtoType::from_primitive(v)?;

        let proto_map = ProtoMap { keys, values };

        ProtoField::Map(proto_map)
      }
      RustType::Option(path) => ProtoField::Optional(ProtoType::from_primitive(path)?),
      RustType::OptionBoxed(path) => {
        return Err(spanned_error!(path, "You seem to be using Option<Box<T>>. If you are using a boxed message, please use message(boxed)"))
      },
      RustType::Boxed(path) => {
        return Err(spanned_error!(path, "You seem to be using Box<T>. If you meant to use a boxed message as a oneof variant, please use message(boxed)"))
      },
      RustType::Vec(path) => ProtoField::Repeated(ProtoType::from_primitive(path)?),
      RustType::Normal(path) => ProtoField::Single(ProtoType::from_primitive(path)?),
    }
  };

  let tag = if let Some(tag) = tag {
    tag
  } else if proto_field.is_oneof() {
    0
  } else {
    return Err(spanned_error!(original_name, "Field tag is missing"));
  };

  Ok(FieldAttrData::Normal(FieldAttrs {
    validator,
    tag,
    options: attributes::ProtoOptions(options),
    name: name.unwrap_or_else(|| ccase!(snake, original_name.to_string())),
    proto_field,
    from_proto,
    into_proto,
  }))
}
