use crate::*;

#[derive(Clone)]
pub struct FieldData {
  pub span: Span,
  pub ident: Ident,
  pub type_info: TypeInfo,
  pub ident_str: String,
  pub is_variant: bool,
  pub tag: Option<i32>,
  pub validator: Option<CallOrClosure>,
  pub options: Option<Expr>,
  pub proto_name: String,
  pub proto_field: ProtoField,
  pub from_proto: Option<PathOrClosure>,
  pub into_proto: Option<PathOrClosure>,
}

pub enum FieldDataKind {
  Ignored {
    ident: Ident,
    from_proto: Option<PathOrClosure>,
  },
  Normal(FieldData),
}

pub fn process_field_data(field: FieldOrVariant) -> Result<FieldDataKind, Error> {
  let mut validator: Option<CallOrClosure> = None;
  let mut tag: Option<i32> = None;
  let mut options: Option<Expr> = None;
  let mut name: Option<String> = None;
  let mut proto_field: Option<ProtoField> = None;
  let mut is_ignored = false;
  let mut from_proto: Option<PathOrClosure> = None;
  let mut into_proto: Option<PathOrClosure> = None;
  let field_ident = field.ident()?.clone();
  let type_info = TypeInfo::from_type(field.get_type()?)?;

  for arg in filter_attributes(field.attributes(), &["proto"])? {
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
            let OneofInfo {
              path,
              tags,
              default,
            } = list.parse_args::<OneofInfo>()?;

            if tags.is_empty() {
              bail!(
                list,
                "Tags for oneofs must be set manually. You can set them with `#[proto(oneof(tags(1, 2, 3)))]`"
              );
            }

            let oneof_path = path
              .get_path_or_fallback(type_info.inner().as_path().as_ref())
              .ok_or(error!(
                list,
                "Failed to infer the path to the oneof. If this is a proxied oneof, use `oneof(proxied)`, otherwise set the path manually with `oneof(MyOneofPath)`"
              ))?;

            proto_field = Some(ProtoField::Oneof {
              path: oneof_path,
              tags,
              default,
            })
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
              return Ok(FieldDataKind::Ignored {
                from_proto,
                ident: field_ident,
              });
            }
          }

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
    return Ok(FieldDataKind::Ignored {
      from_proto,
      ident: field_ident,
    });
  }

  let proto_field = if let Some(mut field) = proto_field {
    if let ProtoField::Single(proto_type) = &mut field
      && type_info.is_option()
    {
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
          return Err(error!(
            inner,
            "You seem to be using Option<Box<T>>. If you are using a boxed message, please use message(boxed)"
          ));
        } else {
          ProtoField::Optional(ProtoType::from_primitive(&inner.require_path()?)?)
        }
      }
      RustType::Box(inner) => {
        return Err(error!(
          inner,
          "You seem to be using Box<T>. If you meant to use a boxed message as a oneof variant, please use message(boxed)"
        ));
      }
      RustType::Vec(inner) => {
        ProtoField::Repeated(ProtoType::from_primitive(&inner.require_path()?)?)
      }
      RustType::Other(inner) => ProtoField::Single(ProtoType::from_primitive(&inner.path)?),
      _ => {
        let path = type_info.as_path().unwrap();

        ProtoField::Single(ProtoType::from_primitive(&path)?)
      }
    }
  };

  Ok(FieldDataKind::Normal(FieldData {
    validator,
    tag,
    options,
    proto_name: name.unwrap_or_else(|| ccase!(snake, field_ident.to_string())),
    proto_field,
    from_proto,
    into_proto,
    span: field.span(),
    ident_str: field_ident.to_string(),
    ident: field_ident,
    is_variant: field.is_variant(),
    type_info,
  }))
}
