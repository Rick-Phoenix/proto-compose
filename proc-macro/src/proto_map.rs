use crate::*;

#[derive(Debug, Clone, Copy)]
pub enum ProtoMapKeys {
  String,
  Bool,
  Int32,
  Int64,
  Sint32,
  Sint64,
  Sfixed32,
  Sfixed64,
  Fixed32,
  Fixed64,
  Uint32,
  Uint64,
}

impl From<ProtoMapKeys> for ProtoType {
  fn from(value: ProtoMapKeys) -> Self {
    match value {
      ProtoMapKeys::String => Self::String,
      ProtoMapKeys::Int32 => Self::Int32,
      ProtoMapKeys::Sint32 => Self::Sint32,
      ProtoMapKeys::Bool => Self::Bool,
      ProtoMapKeys::Int64 => Self::Int64,
      ProtoMapKeys::Sint64 => Self::Sint64,
      ProtoMapKeys::Sfixed32 => Self::Sfixed32,
      ProtoMapKeys::Sfixed64 => Self::Sfixed64,
      ProtoMapKeys::Fixed32 => Self::Fixed32,
      ProtoMapKeys::Fixed64 => Self::Fixed64,
      ProtoMapKeys::Uint32 => Self::Uint32,
      ProtoMapKeys::Uint64 => Self::Uint64,
    }
  }
}

impl ProtoMapKeys {
  pub fn into_type(self) -> ProtoType {
    self.into()
  }
}

impl ProtoMapKeys {
  pub fn from_str(str: &str, span: Span) -> Result<Self, Error> {
    let output = match str {
      "String" | "string" => Self::String,
      "int32" | "i32" => Self::Int32,
      "int64" | "i64" => Self::Int64,
      "uint32" | "u32" => Self::Uint32,
      "uint64" | "u64" => Self::Uint64,
      "bool" => Self::Bool,
      "sint64" => Self::Sint64,
      "sint32" => Self::Sint32,
      "sfixed32" => Self::Sfixed32,
      "sfixed64" => Self::Sfixed64,
      "fixed32" => Self::Fixed32,
      "fixed64" => Self::Fixed64,
      _ => bail_with_span!(span, "Type {str} is not a supported map key primitive"),
    };

    Ok(output)
  }

  pub fn from_path(path: &Path) -> Result<Self, Error> {
    let ident = path.require_ident()?;
    let ident_str = ident.to_string();

    Self::from_str(&ident_str, ident.span())
  }
}

impl Display for ProtoMapKeys {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let pt: ProtoType = (*self).into();

    // Same implementation
    write!(f, "{}", pt.as_prost_map_value())
  }
}

#[derive(Debug, Clone)]
pub struct ProtoMap {
  pub keys: ProtoMapKeys,
  pub values: ProtoType,
  pub is_btree_map: bool,
}

pub fn parse_map_with_context(
  meta: &ParseNestedMeta,
  rust_type: &RustType,
) -> syn::Result<ProtoMap> {
  let mut values: Option<ProtoType> = None;
  let mut keys: Option<ProtoMapKeys> = None;

  let mut idx = 0;

  meta.parse_nested_meta(|meta| {
    match idx {
      0 => keys = Some(ProtoMapKeys::from_path(&meta.path)?),
      1 => {
        let values_type_info = if let RustType::HashMap((_, v)) = rust_type {
          Some(v.as_ref())
        } else if let RustType::BTreeMap((_, v)) = rust_type {
          Some(v.as_ref())
        } else {
          None
        };

        values = Some(ProtoType::from_nested_meta(
          &meta.path.require_ident()?.to_string(),
          &meta,
          values_type_info,
        )?);
      }
      _ => return Err(meta.error("Expected only 2 arguments for keys and values")),
    }
    idx += 1;
    Ok(())
  })?;

  if idx < 2 {
    return Err(meta.error("Expected 2 arguments, for keys and values"));
  }

  let keys = keys.ok_or_else(|| meta.error("Missing key type"))?;
  let values = values.ok_or_else(|| meta.error("Missing values type"))?;

  Ok(ProtoMap {
    keys,
    values,
    is_btree_map: rust_type.is_btree_map(),
  })
}
