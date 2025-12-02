use syn::spanned::Spanned;

use crate::*;

#[derive(Debug, Clone)]
pub enum ProtoMapKeys {
  String,
  Int32,
}

impl From<ProtoMapKeys> for ProtoType {
  fn from(value: ProtoMapKeys) -> Self {
    match value {
      ProtoMapKeys::String => Self::String,
      ProtoMapKeys::Int32 => Self::Int32,
    }
  }
}

impl ProtoMapKeys {
  pub fn validator_target_type(&self) -> TokenStream2 {
    match self {
      ProtoMapKeys::String => quote! { String },
      ProtoMapKeys::Int32 => quote! { i32 },
    }
  }

  pub fn output_proto_type(&self) -> TokenStream2 {
    match self {
      ProtoMapKeys::String => quote! { String },
      ProtoMapKeys::Int32 => quote! { i32 },
    }
  }

  pub fn as_proto_type_trait_target(&self) -> TokenStream2 {
    self.output_proto_type()
  }
}

impl FromStr for ProtoMapKeys {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let output = match s {
      "String" | "string" => Self::String,
      "i32" => Self::Int32,
      _ => return Err(format!("Unrecognized map key {s}")),
    };

    Ok(output)
  }
}

impl ProtoMapKeys {
  pub fn from_path(path: &Path) -> Result<Self, Error> {
    let ident = path.get_ident().ok_or(spanned_error!(
      path,
      format!(
        "Type {} is not a supported map key primitive",
        path.to_token_stream()
      )
    ))?;
    let ident_as_str = ident.to_string();

    Self::from_str(&ident_as_str).map_err(|_| {
      spanned_error!(
        path,
        format!("Type {} is not a supported map key primitive", ident_as_str)
      )
    })
  }
}

impl Display for ProtoMapKeys {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ProtoMapKeys::String => write!(f, "string"),
      ProtoMapKeys::Int32 => write!(f, "int32"),
    }
  }
}

#[derive(Debug, Clone)]
pub struct ProtoMap {
  pub keys: ProtoMapKeys,
  pub values: ProtoType,
}

impl ProtoMap {
  pub fn validator_target_type(&self) -> TokenStream2 {
    let keys = self.keys.validator_target_type();
    let values = self.values.validator_target_type();

    quote! { ProtoMap<#keys, #values> }
  }

  pub fn output_proto_type(&self) -> TokenStream2 {
    let keys = self.keys.output_proto_type();
    let values = self.values.output_proto_type();

    quote! { HashMap<#keys, #values> }
  }

  pub fn as_prost_attr_type(&self) -> TokenStream2 {
    let map_attr = format!("{}, {}", self.keys, self.values.as_prost_map_value());

    quote! { map = #map_attr }
  }

  pub fn as_proto_type_trait_target(&self) -> TokenStream2 {
    let keys = self.keys.as_proto_type_trait_target();
    let values = self.values.as_proto_type_trait_target();

    quote! { HashMap<#keys, #values> }
  }
}

pub fn parse_map_with_context(
  input: syn::parse::ParseStream,
  rust_type: &RustType,
) -> syn::Result<ProtoMap> {
  let metas = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;

  if metas.len() != 2 {
    return Err(input.error("Expected a list of two items"));
  }

  let keys_path = metas.first().unwrap().require_path_only()?;
  let keys = ProtoMapKeys::from_path(keys_path)?;

  let values = match metas.last().take().unwrap() {
    Meta::Path(path) => {
      let ident = path.require_ident()?.to_string();

      let span = path.span();

      let fallback = if let RustType::Map((_, v)) = rust_type {
        Some(v)
      } else {
        return Err(input.error("Not a map type"));
      };

      ProtoType::from_ident(&ident, span, fallback)?
        .ok_or(input.error("Unrecognized map keys type"))?
    }
    Meta::List(list) => {
      let list_ident = ident_string!(list.path);

      let fallback = if let RustType::Map((_, v)) = rust_type {
        Some(v)
      } else {
        return Err(input.error("Not a map type"));
      };

      ProtoType::from_meta_list(&list_ident, list.clone(), fallback)
        .map_err(|e| input.error(e))?
        .ok_or(input.error("Unrecognized map values type"))?
    }
    _ => return Err(input.error("Expected the values to be a list or path")),
  };

  Ok(ProtoMap { keys, values })
}
