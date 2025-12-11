use syn::spanned::Spanned;

use crate::*;

#[derive(Debug, Clone, Default)]
pub enum ProtoType {
  #[default]
  String,
  Bool,
  Bytes,
  Enum(Path),
  Message {
    path: Path,
    is_boxed: bool,
  },
  Int32,
  Sint32,
  Duration,
  Timestamp,
}

impl ProtoType {
  pub fn field_type_tokens(&self) -> TokenStream2 {
    let prefix = quote! { ::proto_types::field_descriptor_proto::Type };
    match self {
      ProtoType::String => quote! { #prefix::String },
      ProtoType::Bool => quote! { #prefix::Bool },
      ProtoType::Bytes => quote! { #prefix::Bytes },
      ProtoType::Enum(_) => quote! { #prefix::Enum },
      ProtoType::Message { .. } => quote! { #prefix::Message },
      ProtoType::Int32 => quote! { #prefix::Int32 },
      ProtoType::Sint32 => quote! { #prefix::Sint32 },
      ProtoType::Duration => quote! { #prefix::Message },
      ProtoType::Timestamp => quote! { #prefix::Message },
    }
  }

  pub fn from_meta(meta: Meta, fallback: Option<&Path>) -> Result<Option<Self>, Error> {
    let output = match meta {
      Meta::List(list) => {
        let ident_str = list.path.require_ident()?.to_string();

        Self::from_meta_list(&ident_str, list, fallback)?
      }
      Meta::Path(path) => {
        let ident_str = path.require_ident()?.to_string();
        let span = path.span();

        Self::from_ident(&ident_str, span, fallback)?
      }
      _ => return Err(spanned_error!(meta, "Expected a path or a metalist")),
    };

    Ok(output)
  }

  pub fn from_meta_list(
    ident_str: &str,
    list: MetaList,
    fallback: Option<&Path>,
  ) -> Result<Option<Self>, Error> {
    let output = match ident_str {
      "message" => {
        let MessageInfo { path, boxed } = list.parse_args::<MessageInfo>()?;

        let path = path.get_path_or_fallback(fallback).ok_or(spanned_error!(
          list,
          "Failed to infer the message path. Please set it manually"
        ))?;

        Self::Message {
          path,
          is_boxed: boxed,
        }
      }
      "enum_" => {
        let path = list.parse_args::<Path>()?;

        Self::Enum(path)
      }
      _ => return Ok(None),
    };

    Ok(Some(output))
  }

  pub fn from_ident(
    ident_str: &str,
    span: Span,
    fallback: Option<&Path>,
  ) -> Result<Option<Self>, Error> {
    let output = match ident_str {
      "string" => Self::String,
      "sint32" => Self::Sint32,
      "message" => {
        let path = fallback
          .ok_or(error!(
            span,
            "Failed to infer the path to the message type. Please set it manually"
          ))?
          .clone();

        Self::Message {
          path,
          is_boxed: false,
        }
      }
      "bytes" => Self::Bytes,
      "bool" => Self::Bool,
      "duration" => Self::Duration,
      "timestamp" => Self::Timestamp,
      "enum_" => {
        let path = fallback
          .ok_or(error!(
            span,
            "Failed to infer the path to the enum type. Please set it manually"
          ))?
          .clone();

        Self::Enum(path)
      }
      _ => return Ok(None),
    };

    Ok(Some(output))
  }

  pub fn as_proto_type_trait_target(&self) -> TokenStream2 {
    match self {
      ProtoType::String => quote! { String },
      ProtoType::Bool => quote! { bool },
      ProtoType::Bytes => quote! { Vec<u8> },
      ProtoType::Enum(path) => quote! { #path },
      ProtoType::Message { path, .. } => quote! { #path },
      ProtoType::Int32 => quote! { i32 },
      ProtoType::Sint32 => quote! { prelude::Sint32 },
      ProtoType::Duration => quote! { proto_types::Duration },
      ProtoType::Timestamp => quote! { proto_types::Timestamp },
    }
  }

  pub fn from_primitive(path: &Path) -> Result<Self, Error> {
    let ident = &path.segments.last().unwrap().ident;
    let ident_str = ident.to_string();

    let output = match ident_str.as_str() {
      "String" => Self::String,
      "bool" => Self::Bool,
      "i32" => Self::Int32,
      "Timestamp" => Self::Timestamp,
      "Duration" => Self::Duration,
      _ => {
        return Err(spanned_error!(
          path,
          format!(
            "Type {} does not correspond to a prost-supported primitive. Please set the protobuf type manually",
            path.to_token_stream()
          )
        ))
      }
    };

    Ok(output)
  }

  pub fn default_from_proto(&self, base_ident: &TokenStream2) -> TokenStream2 {
    match self {
      ProtoType::Enum(_) => quote! { #base_ident.try_into().unwrap_or_default() },
      ProtoType::Message { is_boxed: true, .. } => {
        quote! { Box::new((*#base_ident).into()) }
      }
      _ => quote! { #base_ident.into() },
    }
  }

  pub fn default_into_proto(&self, base_ident: &TokenStream2) -> TokenStream2 {
    match self {
      ProtoType::Message { is_boxed: true, .. } => {
        quote! { Box::new((*#base_ident).into()) }
      }
      _ => quote! { #base_ident.into() },
    }
  }

  pub fn validator_target_type(&self) -> TokenStream2 {
    match self {
      ProtoType::String => quote! { String },
      ProtoType::Bool => quote! { bool },
      ProtoType::Bytes => quote! { Vec<u8> },
      ProtoType::Enum(path) => quote! { #path },
      ProtoType::Message { path, .. } => quote! { #path },
      ProtoType::Int32 => quote! { i32 },
      ProtoType::Sint32 => quote! { ::prelude::Sint32 },
      ProtoType::Duration => quote! { ::proto_types::Duration },
      ProtoType::Timestamp => quote! { ::proto_types::Timestamp },
    }
  }

  pub fn as_prost_map_value(&self) -> Cow<'static, str> {
    match self {
      ProtoType::String => "string".into(),
      ProtoType::Bool => "bool".into(),
      ProtoType::Bytes => "bytes".into(),
      ProtoType::Enum(path) => {
        let path_as_str = path.to_token_stream().to_string();

        format!("enumeration({})", path_as_str).into()
      }
      ProtoType::Message { .. } | ProtoType::Duration | ProtoType::Timestamp => "message".into(),
      ProtoType::Int32 => "int32".into(),
      ProtoType::Sint32 => "sint32".into(),
    }
  }

  pub fn output_proto_type(&self) -> TokenStream2 {
    match self {
      ProtoType::String => quote! { String },
      ProtoType::Bool => quote! { bool },
      ProtoType::Bytes => quote! { Vec<u8> },
      ProtoType::Enum(_) => quote! { i32 },
      ProtoType::Message { path, is_boxed, .. } => {
        if *is_boxed {
          quote! { Box<#path> }
        } else {
          path.to_token_stream()
        }
      }
      ProtoType::Int32 => quote! { i32 },
      ProtoType::Sint32 => quote! { i32 },
      ProtoType::Duration => quote! { proto_types::Duration },
      ProtoType::Timestamp => quote! { proto_types::Timestamp },
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
      ProtoType::Message { is_boxed, .. } => {
        if *is_boxed {
          quote! { message, boxed }
        } else {
          quote! { message }
        }
      }
      ProtoType::Int32 => quote! { int32 },
      ProtoType::Sint32 => quote! { sint32 },
      ProtoType::Duration | ProtoType::Timestamp => quote! { message },
    }
  }

  /// Returns `true` if the proto type is [`Message`].
  ///
  /// [`Message`]: ProtoType::Message
  #[must_use]
  pub fn is_message(&self) -> bool {
    matches!(self, Self::Message { .. })
  }
}
