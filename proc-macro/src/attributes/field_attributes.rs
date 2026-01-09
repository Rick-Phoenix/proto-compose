use crate::*;

#[derive(Clone)]
pub struct ValidatorTokens {
  pub expr: TokenStream2,
  pub is_fallback: bool,
}

impl ToTokens for ValidatorTokens {
  fn to_tokens(&self, tokens: &mut TokenStream2) {
    self.expr.to_tokens(tokens);
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
  pub options: TokensOr<TokenStream2>,
  pub proto_name: String,
  pub proto_field: ProtoField,
  pub from_proto: Option<PathOrClosure>,
  pub into_proto: Option<PathOrClosure>,
  pub deprecated: bool,
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
  pub const fn as_normal(&self) -> Option<&FieldData> {
    if let Self::Normal(v) = self {
      Some(v)
    } else {
      None
    }
  }
}

#[allow(clippy::needless_pass_by_value)]
pub fn process_field_data(field: FieldOrVariant) -> Result<FieldDataKind, Error> {
  let mut validator: Option<ClosureOrExpr> = None;
  let mut tag: Option<i32> = None;
  let mut options = TokensOr::<TokenStream2>::new(|| quote! { vec![] });
  let mut name: Option<String> = None;
  let mut proto_field: Option<ProtoField> = None;
  let mut is_ignored = false;
  let mut from_proto: Option<PathOrClosure> = None;
  let mut into_proto: Option<PathOrClosure> = None;
  let mut deprecated = false;
  let field_ident = field.ident()?.clone();
  let type_info = TypeInfo::from_type(field.get_type()?)?;

  for attr in field.attributes() {
    let ident = if let Some(ident) = attr.path().get_ident() {
      ident.to_string()
    } else {
      continue;
    };

    match ident.as_str() {
      "deprecated" => {
        deprecated = true;
      }
      "proto" => {
        attr.parse_nested_meta(|meta| {
          let ident = meta.path.require_ident()?.to_string();

          match ident.as_str() {
            "deprecated" => {
              let boolean = meta.parse_value::<LitBool>()?;

              deprecated = boolean.value();
            }
            "options" => {
              options.set(meta.expr_value()?.into_token_stream());
            }
            "tag" => {
              tag = Some(meta.expr_value()?.as_int::<i32>()?);
            }
            "name" => {
              name = Some(meta.expr_value()?.as_string()?);
            }
            "validate" => {
              validator = Some(meta.expr_value()?.as_closure_or_expr());
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
              proto_field = Some(ProtoField::from_meta(&ident, &meta, &type_info)?);
            }
          };

          Ok(())
        })?;
      }
      _ => {}
    }
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
            "You seem to be using Option<Box<T>>, but a proto type is not specified. If you are using a boxed message, mark the field as a message"
          ));
        } else {
          ProtoField::Optional(ProtoType::from_primitive(&inner.require_path()?)?)
        }
      }
      RustType::Box(inner) => {
        return Err(error!(
          inner,
          "You seem to be using Box<T>. If you meant to use a boxed message, mark it as a message"
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
        ClosureOrExpr::Expr(expr) => quote! { #expr },

        ClosureOrExpr::Closure(closure) => {
          quote! { <#validator_target_type as ::prelude::ProtoValidator>::validator_from_closure(#closure) }
        }
      }
    });

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
    proto_name: name.unwrap_or_else(|| to_snake_case(&field_ident.to_string())),
    proto_field,
    from_proto,
    into_proto,
    span: field.span(),
    ident_str: field_ident.to_string(),
    ident: field_ident,
    type_info,
    deprecated,
  }))
}
