use crate::*;

#[derive(Clone)]
pub struct FieldAttrs {
  pub tag: i32,
  pub validator: Option<ValidatorExpr>,
  pub options: Option<Expr>,
  pub name: String,
  pub proto_field: ProtoField,
  pub from_proto: Option<PathOrClosure>,
  pub into_proto: Option<PathOrClosure>,
}

#[derive(Clone)]
pub enum FieldAttrData {
  Ignored { from_proto: Option<PathOrClosure> },
  Normal(Box<FieldAttrs>),
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
  let mut options: Option<Expr> = None;
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
          let ident = nv.path.require_ident()?.to_string();

          match ident.as_str() {
            "options" => {
              options = Some(nv.value);
            }

            "validate" => {
              validator = match nv.value {
                Expr::Closure(closure) => Some(ValidatorExpr::Closure(closure)),
                Expr::Call(call) => Some(ValidatorExpr::Call(call)),
                _ => bail!(nv.value, "Expected a closure or a function call"),
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
              tag = Some(extract_i32(&nv.value)?);
            }
            "name" => {
              name = Some(extract_string_lit(&nv.value)?);
            }
            _ => bail!(nv.path, format!("Unknown attribute `{ident}`")),
          };
        }
        Meta::List(list) => {
          let ident = list.path.require_ident()?.to_string();

          match ident.as_str() {
            "oneof" => {
              oneof_attrs.push(list);
            }

            "repeated" => {
              let args = list.parse_args::<Meta>()?;
              let span = args.span();

              let fallback = if let RustType::Vec(path) = rust_type {
                Some(path)
              } else {
                None
              };

              let inner =
                ProtoType::from_meta(args, fallback)?.ok_or(error!(span, "Missing inner type"))?;

              proto_field = Some(ProtoField::Repeated(inner));
            }

            "optional" => {
              let args = list.parse_args::<Meta>()?;
              let span = args.span();

              let fallback = if let RustType::Option(path) = rust_type {
                Some(path)
              } else {
                None
              };

              let inner =
                ProtoType::from_meta(args, fallback)?.ok_or(error!(span, "Missing inner type"))?;

              proto_field = Some(ProtoField::Optional(inner));
            }

            "map" => {
              let parser = |input: ParseStream| parse_map_with_context(input, rust_type);

              let map = parser.parse2(list.tokens)?;

              proto_field = Some(ProtoField::Map(map));
            }

            _ => {
              let list_span = list.span();
              let fallback = rust_type.inner_path();

              if let Some(field_info) = ProtoType::from_meta_list(&ident, list, fallback)? {
                proto_field = Some(ProtoField::Single(field_info));
              } else {
                return Err(error!(list_span, format!("Unknown attribute `{ident}`")));
              }
            }
          };
        }
        Meta::Path(path) => {
          let ident = path.require_ident()?.to_string();

          match ident.as_str() {
            "ignore" => is_ignored = true,
            "oneof" => {}

            _ => {
              let fallback = rust_type.inner_path();
              let span = path.span();

              if let Some(parsed_kind) = ProtoType::from_ident(&ident, span, fallback)? {
                proto_field = Some(ProtoField::Single(parsed_kind));
              } else {
                return Err(error!(span, format!("Unknown attribute `{ident}`")));
              }
            }
          };
        }
      };
    }
  }

  if is_ignored {
    return Ok(FieldAttrData::Ignored { from_proto });
  }

  if !oneof_attrs.is_empty() {
    let mut oneof_path = ItemPath::None;
    let mut oneof_tags: Vec<i32> = Vec::new();
    let mut use_default = false;
    let mut attr_span: Option<Span> = None;

    let fallback = rust_type.inner_path();

    for attr in oneof_attrs {
      let OneofInfo {
        path,
        tags,
        default,
      } = attr.parse_args::<OneofInfo>()?;

      if attr_span.is_none() {
        attr_span = Some(attr.span());
      }

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
      attr_span.unwrap_or_else(Span::call_site),
      "Failed to infer the path to the oneof. Please set it manually"
    ))?;

    proto_field = Some(ProtoField::Oneof {
      path: oneof_path,
      tags: oneof_tags,
      default: use_default,
    })
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

  Ok(FieldAttrData::Normal(Box::new(FieldAttrs {
    validator,
    tag,
    options,
    name: name.unwrap_or_else(|| ccase!(snake, original_name.to_string())),
    proto_field,
    from_proto,
    into_proto,
  })))
}
