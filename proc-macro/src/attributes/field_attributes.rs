use crate::*;

#[derive(Clone)]
pub struct FieldAttrs {
  pub tag: i32,
  pub validator: Option<CallOrClosure>,
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

pub fn process_derive_field_attrs(
  original_name: &Ident,
  type_info: &TypeInfo,
  attrs: &[Attribute],
) -> Result<FieldAttrData, Error> {
  let mut validator: Option<CallOrClosure> = None;
  let mut tag: Option<i32> = None;
  let mut options: Option<Expr> = None;
  let mut name: Option<String> = None;
  let mut proto_field: Option<ProtoField> = None;
  let mut is_ignored = false;
  let mut from_proto: Option<PathOrClosure> = None;
  let mut into_proto: Option<PathOrClosure> = None;

  let mut oneof_attrs: Vec<MetaList> = Vec::new();

  for arg in filter_attributes(attrs, &["proto"])? {
    match arg {
      Meta::NameValue(nv) => {
        let ident = nv.path.require_ident()?.to_string();

        match ident.as_str() {
          "options" => {
            options = Some(nv.value);
          }

          "validate" => {
            validator = Some(nv.value.as_call_or_closure()?);
          }
          "from_proto" => {
            from_proto = Some(nv.value.as_path_or_closure()?);
          }
          "into_proto" => {
            into_proto = Some(nv.value.as_path_or_closure()?);
          }
          "tag" => {
            tag = Some(nv.value.as_int::<i32>()?);
          }
          "name" => {
            name = Some(nv.value.as_string()?);
          }
          _ => bail!(nv.path, "Unknown attribute `{ident}`"),
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

            let fallback = if let RustType::Vec(inner) = type_info.type_.as_ref() {
              inner.as_path()
            } else {
              None
            };

            let inner = ProtoType::from_meta(args, fallback.as_ref())?
              .ok_or(error_with_span!(span, "Missing inner type"))?;

            proto_field = Some(ProtoField::Repeated(inner));
          }

          "optional" => {
            let args = list.parse_args::<Meta>()?;
            let span = args.span();

            let fallback = if let RustType::Option(inner) = type_info.type_.as_ref() {
              inner.as_path()
            } else {
              None
            };

            let inner = ProtoType::from_meta(args, fallback.as_ref())?
              .ok_or(error_with_span!(span, "Missing inner type"))?;

            proto_field = Some(ProtoField::Optional(inner));
          }

          "map" => {
            let parser = |input: ParseStream| parse_map_with_context(input, &type_info.type_);

            let map = parser.parse2(list.tokens)?;

            proto_field = Some(ProtoField::Map(map));
          }

          _ => {
            let list_span = list.span();
            let fallback = type_info.inner().as_path();

            if let Some(field_info) = ProtoType::from_meta_list(&ident, list, fallback.as_ref())? {
              proto_field = Some(ProtoField::Single(field_info));
            } else {
              return Err(error_with_span!(list_span, "Unknown attribute `{ident}`"));
            }
          }
        };
      }
      Meta::Path(path) => {
        let ident = path.require_ident()?.to_string();

        match ident.as_str() {
          "ignore" => {
            is_ignored = true;

            if from_proto.is_some() {
              return Ok(FieldAttrData::Ignored { from_proto });
            }
          }
          "oneof" => {}

          _ => {
            let fallback = type_info.inner().as_path();
            let span = path.span();

            if let Some(parsed_kind) = ProtoType::from_ident(&ident, span, fallback.as_ref())? {
              proto_field = Some(ProtoField::Single(parsed_kind));
            } else {
              return Err(error_with_span!(span, "Unknown attribute `{ident}`"));
            }
          }
        };
      }
    };
  }

  if is_ignored {
    return Ok(FieldAttrData::Ignored { from_proto });
  }

  if !oneof_attrs.is_empty() {
    let mut oneof_path = ItemPathEntry::None;
    let mut oneof_tags: Vec<i32> = Vec::new();
    let mut use_default = false;
    let mut attr_span: Option<Span> = None;

    let fallback = type_info.inner().as_path();

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

    let oneof_path = oneof_path
      .get_path_or_fallback(fallback.as_ref())
      .ok_or(error_with_span!(
        // Just an overly cautious fallback here, the span cannot be empty
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
    if let ProtoField::Single(proto_type) = &mut field && type_info.is_option() {
      let inner = std::mem::take(proto_type);

      field = ProtoField::Optional(inner);
    }

    field
  } else {
    match type_info.type_.as_ref() {
      RustType::HashMap((k, v)) => {
        let keys = ProtoMapKeys::from_path(&k.require_path()?)?;

        let values = ProtoType::from_primitive(&v.require_path()?)?;

        let proto_map = ProtoMap { keys, values };

        ProtoField::Map(proto_map)
      }
      RustType::Option(inner) => {
        if inner.is_box() {
        return Err(error!(inner, "You seem to be using Option<Box<T>>. If you are using a boxed message, please use message(boxed)"))
        } else {
          ProtoField::Optional(ProtoType::from_primitive(&inner.require_path()?)?)
        }
      },
      RustType::Box(inner) => {
        return Err(error!(inner, "You seem to be using Box<T>. If you meant to use a boxed message as a oneof variant, please use message(boxed)"))
      },
      RustType::Vec(inner) => ProtoField::Repeated(ProtoType::from_primitive(&inner.require_path()?)?),
      RustType::Other(inner) => ProtoField::Single(ProtoType::from_primitive(&inner.path)?),
      _ => bail!(type_info, "Failed to infer the protobuf type. Please set it manually")
    }
  };

  let tag = if let Some(tag) = tag {
    tag
  } else if proto_field.is_oneof() {
    0
  } else {
    return Err(error!(original_name, "Field tag is missing"));
  };

  let name = name.unwrap_or_else(|| ccase!(snake, original_name.to_string()));

  Ok(FieldAttrData::Normal(Box::new(FieldAttrs {
    validator,
    tag,
    options,
    name,
    proto_field,
    from_proto,
    into_proto,
  })))
}
