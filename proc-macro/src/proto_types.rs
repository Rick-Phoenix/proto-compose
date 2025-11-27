use std::str::FromStr;

use crate::*;

#[derive(Debug, Clone)]
pub enum ProtoType {
  String,
  Bool,
  Bytes,
  Enum(Path),
  Message(Path),
  Int32,
  Map(ProtoMap),
  Sint32,
}

impl ProtoType {
  pub fn from_primitive(path: &Path) -> Result<Self, Error> {
    let ident = path.require_ident()?;
    let ident_str = ident.to_string();

    let output = match ident_str.as_str() {
      "String" => Self::String,
      "bool" => Self::Bool,
      "i32" => Self::Int32,
      _ => {
        return Err(spanned_error!(
          path,
          format!(
            "Type {} does not correspond to a prost-supported primitive. Use the specific attributes if you meant to use an enum or message",
            path.to_token_stream()
          )
        ))
      }
    };

    Ok(output)
  }

  pub fn validator_target_type(&self) -> TokenStream2 {
    match self {
      ProtoType::String => quote! { String },
      ProtoType::Bool => quote! { bool },
      ProtoType::Bytes => quote! { Vec<u8> },
      ProtoType::Enum(_) => quote! { GenericProtoEnum },
      ProtoType::Message(_) => quote! { GenericMessage },
      ProtoType::Int32 => quote! { i32 },
      ProtoType::Map(map) => map.validator_target_type(),
      ProtoType::Sint32 => quote! { Sint32 },
    }
  }

  pub fn as_proto_type_trait_target(&self) -> TokenStream2 {
    match self {
      ProtoType::String => quote! { String },
      ProtoType::Bool => quote! { bool },
      ProtoType::Bytes => quote! { Vec<u8> },
      ProtoType::Enum(path) => quote! { #path },
      ProtoType::Message(path) => quote! { #path },
      ProtoType::Int32 => quote! { i32 },
      ProtoType::Map(map) => map.as_proto_type_trait_target(),
      ProtoType::Sint32 => quote! { i32 },
    }
  }

  pub fn output_proto_type(&self) -> TokenStream2 {
    match self {
      ProtoType::String => quote! { String },
      ProtoType::Bool => quote! { bool },
      ProtoType::Bytes => quote! { Vec<u8> },
      ProtoType::Enum(_) => quote! { i32 },
      ProtoType::Message(path) => {
        let path_with_proto_suffix = append_proto_ident(path.clone());

        path_with_proto_suffix.to_token_stream()
      }
      ProtoType::Int32 => quote! { i32 },
      ProtoType::Map(map) => map.output_proto_type(),
      ProtoType::Sint32 => quote! { i32 },
    }
  }

  pub fn as_prost_attr_type(&self) -> TokenStream2 {
    match self {
      ProtoType::String => quote! { string },
      ProtoType::Bool => quote! { bool },
      ProtoType::Bytes => quote! { bytes = "bytes" },
      ProtoType::Enum(path) => {
        let path_as_str = path.to_token_stream().to_string();

        quote! { enumeration = #path_as_str }
      }
      ProtoType::Message(_) => quote! { message },
      ProtoType::Int32 => quote! { int32 },
      ProtoType::Map(map) => map.as_prost_attr_type(),
      ProtoType::Sint32 => quote! { sint32 },
    }
  }
}
