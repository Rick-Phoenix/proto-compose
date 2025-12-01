use std::str::FromStr;

use syn::spanned::Spanned;

use crate::*;

#[derive(Debug, Clone)]
pub enum ProtoType {
  String,
  Bool,
  Bytes,
  Enum(Path),
  Message {
    path: Path,
    boxed: bool,
  },
  Int32,
  Map(ProtoMap),
  Sint32,
  Oneof {
    path: Path,
    tags: Vec<i32>,
    default: bool,
    is_proxied: bool,
  },
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

  pub fn default_from_proto(&self) -> TokenStream2 {
    match self {
      ProtoType::Enum(_) => quote! { try_into().unwrap_or_default() },
      _ => quote! { into() },
    }
  }

  pub fn validator_target_type(&self) -> TokenStream2 {
    match self {
      ProtoType::String => quote! { String },
      ProtoType::Bool => quote! { bool },
      ProtoType::Bytes => quote! { Vec<u8> },
      ProtoType::Enum(_) => quote! { GenericProtoEnum },
      ProtoType::Message { .. } => quote! { GenericMessage },
      ProtoType::Int32 => quote! { i32 },
      ProtoType::Map(map) => map.validator_target_type(),
      ProtoType::Sint32 => quote! { Sint32 },
      ProtoType::Oneof { .. } => quote! {},
    }
  }

  pub fn as_proto_type_trait_target(&self) -> TokenStream2 {
    match self {
      ProtoType::String => quote! { String },
      ProtoType::Bool => quote! { bool },
      ProtoType::Bytes => quote! { Vec<u8> },
      ProtoType::Enum(path) => quote! { #path },
      ProtoType::Message { path, .. } => quote! { #path },
      ProtoType::Int32 => quote! { i32 },
      ProtoType::Map(map) => map.as_proto_type_trait_target(),
      ProtoType::Sint32 => quote! { i32 },
      ProtoType::Oneof { .. } => quote! {},
    }
  }

  pub fn output_proto_type(&self) -> TokenStream2 {
    match self {
      ProtoType::String => quote! { String },
      ProtoType::Bool => quote! { bool },
      ProtoType::Bytes => quote! { Vec<u8> },
      ProtoType::Enum(_) => quote! { i32 },
      ProtoType::Message { path, .. } => path.to_token_stream(),
      ProtoType::Int32 => quote! { i32 },
      ProtoType::Map(map) => map.output_proto_type(),
      ProtoType::Sint32 => quote! { i32 },
      ProtoType::Oneof { path, .. } => path.to_token_stream(),
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
      ProtoType::Message { .. } => quote! { message },
      ProtoType::Int32 => quote! { int32 },
      ProtoType::Map(map) => map.as_prost_attr_type(),
      ProtoType::Sint32 => quote! { sint32 },
      ProtoType::Oneof { .. } => {
        todo!()
      }
    }
  }
}

pub fn extract_proto_type(
  rust_type: &RustType,
  field_type: ProtoFieldKind,
  field_ty: &Type,
) -> Result<ProtoType, Error> {
  let output = match field_type {
    ProtoFieldKind::Oneof(OneofInfo {
      path,
      tags,
      default,
    }) => {
      let oneof_path = match &path {
        ItemPath::Path(path) => path.clone(),

        _ => {
          let inner_type = rust_type
            .inner_path()
            .ok_or(spanned_error!(
            field_ty,
            // SHould refine this for oneofs
            "Failed to extract the inner type. Expected a type, or a type wrapped in Option or Vec"
          ))?
            .clone();

          if path.is_suffixed() {
            append_proto_ident(inner_type)
          } else {
            inner_type
          }
        }
      };

      ProtoType::Oneof {
        path: oneof_path,
        tags,
        default,
        is_proxied: !path.is_none(),
      }
    }
    ProtoFieldKind::Enum(path) => {
      // Handle the errors here and just say it can't be used for a map
      let enum_path = if let Some(path) = path {
        path
      } else {
        rust_type
          .inner_path()
          .ok_or(spanned_error!(
            field_ty,
            "Failed to extract the inner type. Expected a type, or a type wrapped in Option or Vec"
          ))?
          .clone()
      };

      ProtoType::Enum(enum_path)
    }
    ProtoFieldKind::Message(MessageInfo { path, boxed }) => {
      let msg_path = if let ItemPath::Path(path) = path {
        path
      } else {
        let inner_type = rust_type
          .inner_path()
          .ok_or(spanned_error!(
            field_ty,
            "Failed to extract the inner type. Expected a type, or a type wrapped in Option or Vec"
          ))?
          .clone();

        if path.is_suffixed() {
          append_proto_ident(inner_type)
        } else {
          inner_type
        }
      };

      ProtoType::Message {
        path: msg_path,
        boxed,
      }
    }
    ProtoFieldKind::Map(proto_map) => ProtoType::Map(set_map_proto_type(proto_map, rust_type)?),
    // No manually set type, let's try to infer it as a primitive
    // maybe use the larger error for any of these
    _ => match rust_type {
      RustType::Option(path) => ProtoType::from_primitive(path)?,
      RustType::BoxedMsg(path) => ProtoType::from_primitive(path)?,
      RustType::Vec(path) => ProtoType::from_primitive(path)?,
      RustType::Normal(path) => ProtoType::from_primitive(path)?,
      RustType::BoxedOneofVariant(path) => ProtoType::from_primitive(path)?,
      RustType::Map((k, v)) => {
        let keys = ProtoMapKeys::from_path(k)?;
        let values = ProtoMapValues::from_path(v).map_err(|_| spanned_error!(v, format!("Unrecognized proto map value type {}. If you meant to use an enum or a message, use the attribute", v.to_token_stream())))?;

        let proto_map = ProtoMap { keys, values };

        ProtoType::Map(set_map_proto_type(proto_map, rust_type)?)
      }
    },
  };

  Ok(output)
}
