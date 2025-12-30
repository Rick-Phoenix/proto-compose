use crate::*;

#[derive(Clone)]
pub struct ValidatorTokens {
  pub expr: TokenStream2,
  pub is_fallback: bool,
}

impl ToTokens for ValidatorTokens {
  fn to_tokens(&self, tokens: &mut TokenStream2) {
    tokens.extend(self.expr.to_token_stream());
  }
}

#[derive(Clone)]
pub struct FieldData {
  pub span: Span,
  pub ident: Ident,
  pub type_info: TypeInfo,
  pub ident_str: String,
  pub tag: Option<i32>,
  pub validator: Option<ValidatorTokens>,
  pub options: Option<Expr>,
  pub proto_name: String,
  pub proto_field: ProtoField,
  pub from_proto: Option<PathOrClosure>,
  pub into_proto: Option<PathOrClosure>,
}

// No sense in boxing since it's the most common path
#[allow(clippy::large_enum_variant)]
pub enum FieldDataKind {
  Ignored {
    ident: Ident,
    from_proto: Option<PathOrClosure>,
  },
  Normal(FieldData),
}

impl FieldDataKind {
  pub fn as_normal(&self) -> Option<&FieldData> {
    if let Self::Normal(v) = self {
      Some(v)
    } else {
      None
    }
  }
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

  parse_filtered_attrs(field.attributes(), &["proto"], |meta| {
    let ident = meta.path.require_ident()?.to_string();

    match ident.as_str() {
      "options" => {
        options = Some(meta.expr_value()?);
      }
      "tag" => {
        tag = Some(meta.expr_value()?.as_int::<i32>()?);
      }
      "name" => {
        name = Some(meta.expr_value()?.as_string()?);
      }
      "validate" => {
        validator = Some(meta.expr_value()?.as_call_or_closure()?);
      }
      "from_proto" => {
        from_proto = Some(meta.expr_value()?.as_path_or_closure()?);
      }
      "into_proto" => {
        into_proto = Some(meta.expr_value()?.as_path_or_closure()?);
      }
      "ignore" => {
        is_ignored = true;
      }

      _ => {
        proto_field = Some(ProtoField::from_meta(&ident, meta, &type_info)?);
      }
    };

    Ok(())
  })?;

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

  let validator_expr = validator.as_ref().map(|validator|  {
      let validator_target_type = proto_field.validator_target_type();

      match validator {
        CallOrClosure::Call(call) => quote! { #call.build_validator() },

        CallOrClosure::Closure(closure) => {
          quote! { <#validator_target_type as ::prelude::ProtoValidator>::validator_from_closure(#closure) }
        }
      }
    });

  // I think it's clearer this way
  #[allow(clippy::manual_map)]
  let validator = if let Some(expr) = validator_expr {
    Some(ValidatorTokens {
      expr,
      is_fallback: false,
    })
  } else if let Some(expr) = proto_field.default_validator_expr() {
    Some(ValidatorTokens {
      expr,
      is_fallback: true,
    })
  } else {
    None
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
    type_info,
  }))
}
