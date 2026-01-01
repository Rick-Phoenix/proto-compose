use crate::*;

#[derive(Clone)]
pub struct ExtensionFieldAttrs {
  pub tag: Option<i32>,
  pub options: TokensOr<TokenStream2>,
  pub proto_name: String,
  pub proto_field: ProtoField,
}

pub fn process_extension_field_attrs(field: &mut Field) -> Result<ExtensionFieldAttrs, Error> {
  let mut tag: Option<i32> = None;
  let mut options = TokensOr::<TokenStream2>::new(|| quote! { vec![] });
  let mut name: Option<String> = None;
  let mut proto_field: Option<ProtoField> = None;

  let field_ident = field.require_ident()?.clone();
  let type_info = TypeInfo::from_type(&field.ty)?;

  parse_filtered_attrs(&field.attrs, &["proto"], |meta| {
    let ident = meta.path.require_ident()?.to_string();

    match ident.as_str() {
      "options" => {
        options.set(meta.expr_value()?.into_token_stream());
      }
      "tag" => {
        tag = Some(meta.expr_value()?.as_int::<i32>()?);
      }
      "name" => {
        name = Some(meta.expr_value()?.as_string()?);
      }

      _ => {
        proto_field = Some(ProtoField::from_meta(&ident, meta, &type_info)?);
      }
    };

    Ok(())
  })?;

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

  Ok(ExtensionFieldAttrs {
    tag,
    options,
    proto_name: name.unwrap_or_else(|| ccase!(snake, field_ident.to_string())),
    proto_field,
  })
}
