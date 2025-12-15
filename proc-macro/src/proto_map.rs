use syn::spanned::Spanned;

use crate::*;

#[derive(Debug, Clone)]
pub enum ProtoMapKeys {
  String,
  Int32,
  Sint32,
}

impl From<ProtoMapKeys> for ProtoType {
  fn from(value: ProtoMapKeys) -> Self {
    match value {
      ProtoMapKeys::String => Self::String,
      ProtoMapKeys::Int32 => Self::Int32,
      ProtoMapKeys::Sint32 => Self::Sint32,
    }
  }
}

impl ProtoMapKeys {
  pub fn validator_target_type(&self) -> TokenStream2 {
    match self {
      ProtoMapKeys::String => quote! { String },
      ProtoMapKeys::Int32 => quote! { i32 },
      ProtoMapKeys::Sint32 => quote! { ::prelude::Sint32 },
    }
  }

  pub fn output_proto_type(&self) -> TokenStream2 {
    match self {
      ProtoMapKeys::String => quote! { String },
      ProtoMapKeys::Int32 | ProtoMapKeys::Sint32 => quote! { i32 },
    }
  }

  pub fn as_proto_type_trait_target(&self) -> TokenStream2 {
    match self {
      ProtoMapKeys::String => quote! { String },
      ProtoMapKeys::Int32 => quote! { i32 },
      ProtoMapKeys::Sint32 => quote! { ::prelude::Sint32 },
    }
  }
}

impl ProtoMapKeys {
  pub fn from_path(path: &Path) -> Result<Self, Error> {
    let ident = path.require_ident()?;
    let ident_str = ident.to_string();

    let output = match ident_str.as_str() {
      "String" | "string" => Self::String,
      "int32" | "i32" => Self::Int32,
      "sint32" => Self::Sint32,
      _ => bail!(
        ident,
        format!("Type {} is not a supported map key primitive", ident_str)
      ),
    };

    Ok(output)
  }
}

impl Display for ProtoMapKeys {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ProtoMapKeys::String => write!(f, "string"),
      ProtoMapKeys::Int32 => write!(f, "int32"),
      ProtoMapKeys::Sint32 => write!(f, "sint32"),
    }
  }
}

#[derive(Debug, Clone)]
pub struct ProtoMap {
  pub keys: ProtoMapKeys,
  pub values: ProtoType,
}

impl ProtoMap {
  pub fn as_proto_type_trait_target(&self) -> TokenStream2 {
    let keys = self.keys.as_proto_type_trait_target();
    let values = self.values.as_proto_type_trait_target();

    quote! { ::prelude::ProtoMap<#keys, #values> }
  }

  pub fn validator_target_type(&self) -> TokenStream2 {
    let keys = self.keys.validator_target_type();
    let values = self.values.validator_target_type();

    quote! { ::prelude::ProtoMap<#keys, #values> }
  }

  pub fn output_proto_type(&self) -> TokenStream2 {
    let keys = self.keys.output_proto_type();
    let values = self.values.output_proto_type();

    quote! { std::collections::HashMap<#keys, #values> }
  }

  pub fn as_prost_attr_type(&self) -> TokenStream2 {
    let map_attr = format!("{}, {}", self.keys, self.values.as_prost_map_value());

    quote! { map = #map_attr }
  }
}

pub fn parse_map_with_context(
  input: syn::parse::ParseStream,
  rust_type: &RustType,
) -> syn::Result<ProtoMap> {
  let mut metas = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;

  if metas.len() != 2 {
    bail!(metas, "Expected a list of two items");
  }

  let values = match metas.pop().unwrap().into_value() {
    Meta::Path(path) => {
      let ident = path.require_ident()?.to_string();
      let span = path.span();

      let fallback = if let RustType::HashMap((_, v)) = rust_type {
        v.as_path()
      } else {
        None
      };

      ProtoType::from_ident(&ident, span, fallback.as_ref())?
        .ok_or(error!(span, "Unrecognized map keys type"))?
    }
    Meta::List(list) => {
      let list_ident = ident_string!(list.path);
      let span = list.span();

      let fallback = if let RustType::HashMap((_, v)) = rust_type {
        v.as_path()
      } else {
        None
      };

      ProtoType::from_meta_list(&list_ident, list, fallback.as_ref())
        .map_err(|e| input.error(e))?
        .ok_or(error!(span, "Unrecognized map values type"))?
    }
    Meta::NameValue(nv) => bail!(nv, "Expected the values to be a list or path"),
  };

  let keys_input = metas.pop().unwrap().into_value();
  let keys_path = keys_input.require_path_only()?;
  let keys = ProtoMapKeys::from_path(keys_path)?;

  Ok(ProtoMap { keys, values })
}
