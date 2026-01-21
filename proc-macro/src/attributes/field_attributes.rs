use std::ops::Deref;

use crate::*;

#[derive(Clone, Copy)]
pub enum ValidatorKind {
  Closure,
  Reflection,
  Custom,
  Default,
  DefaultOneof,
  RequiredOneof,
}

impl ValidatorKind {
  /// Returns `true` if the validator kind is [`Default`].
  ///
  /// [`Default`]: ValidatorKind::Default
  #[must_use]
  pub const fn is_default(self) -> bool {
    matches!(self, Self::Default | Self::DefaultOneof)
  }

  pub const fn should_be_cached(self) -> bool {
    matches!(self, Self::Closure | Self::Reflection | Self::Default)
  }

  /// Returns `true` if the validator kind is [`Custom`].
  ///
  /// [`Custom`]: ValidatorKind::Custom
  #[must_use]
  pub const fn is_custom(self) -> bool {
    matches!(self, Self::Custom)
  }

  /// Returns `true` if the validator kind is [`Closure`].
  ///
  /// [`Closure`]: ValidatorKind::Closure
  #[must_use]
  pub const fn is_closure(self) -> bool {
    matches!(self, Self::Closure)
  }
}

#[derive(Clone)]
pub struct ValidatorTokens {
  pub expr: TokenStream2,
  pub kind: ValidatorKind,
  pub span: Span,
}

#[derive(Clone, Default)]
pub struct Validators {
  pub validators: Vec<ValidatorTokens>,
}

impl<'a> IntoIterator for &'a Validators {
  type Item = &'a ValidatorTokens;
  type IntoIter = std::slice::Iter<'a, ValidatorTokens>;

  fn into_iter(self) -> Self::IntoIter {
    self.validators.iter()
  }
}

impl Deref for Validators {
  type Target = [ValidatorTokens];

  fn deref(&self) -> &Self::Target {
    &self.validators
  }
}

impl Validators {
  pub fn span(&self) -> Span {
    self
      .validators
      .first()
      .as_ref()
      .map_or_else(|| Span::call_site(), |v| v.span)
  }

  pub fn from_single(validator: ValidatorTokens) -> Self {
    Self {
      validators: vec![validator],
    }
  }
  pub fn iter(&self) -> std::slice::Iter<'_, ValidatorTokens> {
    self.validators.iter()
  }

  pub fn adjust_closures(&mut self, proto_field: &ProtoField) {
    for validator in &mut self.validators {
      if validator.kind.is_closure() {
        let validator_target_type = proto_field.validator_target_type(validator.span);

        validator.expr = quote_spanned! {validator.span=> <#validator_target_type as ::prelude::ProtoValidator>::validator_from_closure(#validator) };
      }
    }
  }
}

impl Parse for ValidatorTokens {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let validator: Expr = input.parse()?;

    let kind = match &validator {
      Expr::Closure(_) => ValidatorKind::Closure,
      _ => ValidatorKind::Custom,
    };

    Ok(Self {
      span: validator.span(),
      kind,
      expr: validator.into_token_stream(),
    })
  }
}

impl Parse for Validators {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let mut validators: Vec<ValidatorTokens> = Vec::new();

    if input.peek(token::Bracket) {
      let content;
      bracketed!(content in input);

      while !content.is_empty() {
        validators.push(content.parse::<ValidatorTokens>()?);

        if content.is_empty() {
          break;
        }

        let _comma: token::Comma = content.parse()?;
      }
    } else {
      validators.push(input.parse::<ValidatorTokens>()?);
    }

    Ok(Self { validators })
  }
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
  pub tag: Option<ParsedNum>,
  pub validators: Validators,
  pub options: TokensOr<TokenStream2>,
  pub proto_name: String,
  pub proto_field: ProtoField,
  pub from_proto: Option<PathOrClosure>,
  pub into_proto: Option<PathOrClosure>,
  pub deprecated: bool,
}

impl FieldData {
  pub const fn has_custom_conversions(&self) -> bool {
    self.from_proto.is_some() && self.into_proto.is_some()
  }
}

// No point in boxing `Normal` since it's the most common path
#[allow(clippy::large_enum_variant)]
pub enum FieldDataKind {
  Ignored {
    ident: Ident,
    from_proto: Option<PathOrClosure>,
    // This is only for converting oneof variants
    // that are ignored in the proto enum
    into_proto: Option<PathOrClosure>,
  },
  Normal(FieldData),
}

impl FieldDataKind {
  pub const fn ident(&self) -> &Ident {
    match self {
      Self::Ignored { ident, .. } => ident,
      Self::Normal(field_data) => &field_data.ident,
    }
  }

  pub const fn as_normal(&self) -> Option<&FieldData> {
    if let Self::Normal(v) = self {
      Some(v)
    } else {
      None
    }
  }

  /// Returns `true` if the field data kind is [`Ignored`].
  ///
  /// [`Ignored`]: FieldDataKind::Ignored
  #[must_use]
  pub const fn is_ignored(&self) -> bool {
    matches!(self, Self::Ignored { .. })
  }
}

#[allow(clippy::needless_pass_by_value)]
pub fn process_field_data(field: FieldOrVariant) -> Result<FieldDataKind, Error> {
  let field_span = field.ident()?.span();

  let mut validators = Validators::default();
  let mut tag: Option<ParsedNum> = None;
  let mut options = TokensOr::<TokenStream2>::vec();
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
              options.span = meta.input.span();
              options.set(meta.expr_value()?.into_token_stream());
            }
            "tag" => {
              tag = Some(meta.parse_value::<ParsedNum>()?);
            }
            "name" => {
              name = Some(meta.expr_value()?.as_string()?);
            }
            "validate" => {
              validators = meta.parse_value::<Validators>()?;
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
      into_proto,
    });
  }

  let proto_field = if let Some(mut field) = proto_field {
    // We try to infer if a field is `Option` or `Vec` but
    // wasn't explicitely marked as optional/repeated
    if let ProtoField::Single(proto_type) = &mut field
      && (type_info.is_option() || type_info.is_vec())
    {
      let inner = std::mem::take(proto_type);

      field = if type_info.is_option() {
        ProtoField::Optional(inner)
      } else {
        ProtoField::Repeated(inner)
      };
    }

    field
  } else {
    // Field received no type information at all, we try to do some basic inference
    match type_info.type_.as_ref() {
      RustType::HashMap((k, v)) | RustType::BTreeMap((k, v)) => {
        let keys = ProtoMapKeys::from_path(&k.require_path()?)?;

        let values = ProtoType::from_primitive(&v.require_path()?)?;

        let proto_map = ProtoMap {
          keys,
          values,
          is_btree_map: type_info.is_btree_map(),
        };

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
        let path = type_info.as_path().ok_or_else(|| {
          error_with_span!(
            field_span,
            "Failed to infer the protobuf type. Please set it manually"
          )
        })?;

        ProtoField::Single(ProtoType::from_primitive(&path)?)
      }
    }
  };

  if !validators.validators.is_empty() {
    validators.adjust_closures(&proto_field);
  } else if let Some(default) = proto_field.default_validator_expr(field_span) {
    validators.validators.push(default);
  }

  let proto_name = name.unwrap_or_else(|| {
    if field.is_variant() {
      to_snake_case(&field_ident.to_string())
    } else {
      rust_ident_to_proto_name(&field_ident)
    }
  });

  Ok(FieldDataKind::Normal(FieldData {
    validators,
    tag,
    options,
    proto_name,
    proto_field,
    from_proto,
    into_proto,
    span: field_span,
    ident_str: field_ident.to_string(),
    ident: field_ident,
    type_info,
    deprecated,
  }))
}

pub fn rust_ident_to_proto_name(rust_ident: &Ident) -> String {
  let str = rust_ident.to_string();

  if let Some(escaped) = str
    .strip_prefix("r#")
    .or_else(|| str.strip_suffix("_"))
  {
    escaped.to_string()
  } else {
    str
  }
}
