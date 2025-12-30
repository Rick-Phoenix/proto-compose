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
  Float,
  Double,
  Int32,
  Int64,
  Uint32,
  Uint64,
  Sint32,
  Sint64,
  Fixed32,
  Fixed64,
  Sfixed32,
  Sfixed64,
  Duration,
  Timestamp,
}

impl ProtoType {
  pub fn descriptor_type_tokens(&self) -> TokenStream2 {
    let prefix = quote! { ::proto_types::field_descriptor_proto::Type };

    match self {
      ProtoType::Uint32 => quote! { #prefix::Uint32 },
      ProtoType::String => quote! { #prefix::String },
      ProtoType::Bool => quote! { #prefix::Bool },
      ProtoType::Bytes => quote! { #prefix::Bytes },
      ProtoType::Enum(_) => quote! { #prefix::Enum },
      ProtoType::Message { .. } => quote! { #prefix::Message },
      ProtoType::Int32 => quote! { #prefix::Int32 },
      ProtoType::Sint32 => quote! { #prefix::Sint32 },
      ProtoType::Duration => quote! { #prefix::Message },
      ProtoType::Timestamp => quote! { #prefix::Message },
      ProtoType::Float => quote! { #prefix::Float },
      ProtoType::Double => quote! { #prefix::Double },
      ProtoType::Int64 => quote! { #prefix::Int64  },
      ProtoType::Uint64 => quote! { #prefix::Uint64  },
      ProtoType::Sint64 => quote! { #prefix::Sint64  },
      ProtoType::Fixed32 => quote! { #prefix::Fixed32  },
      ProtoType::Fixed64 => quote! { #prefix::Fixed64  },
      ProtoType::Sfixed32 => quote! { #prefix::Sfixed32  },
      ProtoType::Sfixed64 => quote! { #prefix::Sfixed64  },
    }
  }

  pub fn from_nested_meta(
    ident_str: &str,
    meta: ParseNestedMeta,
    fallback: Option<&Path>,
  ) -> Result<Self, Error> {
    let output = match meta.meta_type() {
      MetaType::Path => {
        let span = meta.path.span();

        Self::from_ident(ident_str, span, fallback)?
      }
      MetaType::List => Self::from_meta_list(ident_str, meta, fallback)?,
      MetaType::NameValue => return Err(meta.error("Expected a path or a metalist")),
    };

    Ok(output)
  }

  pub fn from_meta_list(
    ident_str: &str,
    meta: ParseNestedMeta,
    fallback: Option<&Path>,
  ) -> Result<Self, Error> {
    let output = match ident_str {
      "message" => {
        let MessageInfo { path, boxed } = meta.parse_list::<MessageInfo>()?;

        let path = path
          .get_path_or_fallback(fallback)
          .ok_or(meta.error("Failed to infer the message path. Please set it manually"))?;

        Self::Message {
          path,
          is_boxed: boxed,
        }
      }
      "enum_" => {
        let path = meta.parse_list::<Path>()?;

        Self::Enum(path)
      }
      _ => return Err(meta.error("Unknown protobuf type")),
    };

    Ok(output)
  }

  pub fn from_ident(ident_str: &str, span: Span, fallback: Option<&Path>) -> Result<Self, Error> {
    let output = match ident_str {
      "string" => Self::String,
      "int32" => Self::Int32,
      "int64" => Self::Int64,
      "sint32" => Self::Sint32,
      "sint64" => Self::Sint64,
      "sfixed32" => Self::Sfixed32,
      "sfixed64" => Self::Sfixed64,
      "fixed32" => Self::Fixed32,
      "fixed64" => Self::Fixed64,
      "uint32" => Self::Uint32,
      "uint64" => Self::Uint64,
      "float" => Self::Float,
      "double" => Self::Double,
      "message" => {
        let path = fallback
          .ok_or(error_with_span!(
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
          .ok_or(error_with_span!(
            span,
            "Failed to infer the path to the enum type. Please set it manually"
          ))?
          .clone();

        Self::Enum(path)
      }
      _ => return Err(error_with_span!(span, "Unknown protobuf type {ident_str}")),
    };

    Ok(output)
  }

  pub fn field_proto_type_tokens(&self) -> TokenStream2 {
    match self {
      ProtoType::String => quote! { String },
      ProtoType::Bool => quote! { bool },
      ProtoType::Bytes => quote! { ::bytes::Bytes },
      ProtoType::Enum(path) => quote! { #path },
      ProtoType::Message { path, .. } => quote! { #path },
      ProtoType::Int32 => quote! { i32 },
      ProtoType::Sint32 => quote! { prelude::Sint32 },
      ProtoType::Duration => quote! { proto_types::Duration },
      ProtoType::Timestamp => quote! { proto_types::Timestamp },
      ProtoType::Uint32 => quote! { u32 },
      ProtoType::Float => quote! { f32 },
      ProtoType::Double => quote! { f64 },
      ProtoType::Int64 => quote! { i64  },
      ProtoType::Uint64 => quote! { u64 },
      ProtoType::Sint64 => quote! { prelude::Sint64  },
      ProtoType::Fixed32 => quote! { prelude::Fixed32  },
      ProtoType::Fixed64 => quote! { prelude::Fixed64  },
      ProtoType::Sfixed32 => quote! { prelude::Sfixed32  },
      ProtoType::Sfixed64 => quote! { prelude::Sfixed64  },
    }
  }

  pub fn from_primitive(path: &Path) -> Result<Self, Error> {
    let ident = &path.segments.last().unwrap().ident;
    let ident_str = ident.to_string();

    let output = match ident_str.as_str() {
      "String" => Self::String,
      "bool" => Self::Bool,
      "i32" => Self::Int32,
      "u32" => Self::Uint32,
      "Timestamp" => Self::Timestamp,
      "Duration" => Self::Duration,
      "f32" => Self::Float,
      _ => {
        return Err(error!(
          path,
          "Type {} does not correspond to a prost-supported primitive. Please set the protobuf type manually",
          path.to_token_stream()
        ));
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
      ProtoType::Bytes => quote! { ::bytes::Bytes },
      ProtoType::Enum(path) => quote! { #path },
      ProtoType::Message { path, .. } => quote! { #path },
      ProtoType::Int32 => quote! { i32 },
      ProtoType::Sint32 => quote! { ::prelude::Sint32 },
      ProtoType::Duration => quote! { ::proto_types::Duration },
      ProtoType::Timestamp => quote! { ::proto_types::Timestamp },
      ProtoType::Uint32 => quote! { u32 },
      ProtoType::Float => quote! { f32 },
      ProtoType::Double => quote! { f64 },
      ProtoType::Int64 => quote! { i64  },
      ProtoType::Uint64 => quote! { u64 },
      ProtoType::Sint64 => quote! { prelude::Sint64  },
      ProtoType::Fixed32 => quote! { prelude::Fixed32  },
      ProtoType::Fixed64 => quote! { prelude::Fixed64  },
      ProtoType::Sfixed32 => quote! { prelude::Sfixed32  },
      ProtoType::Sfixed64 => quote! { prelude::Sfixed64  },
    }
  }

  pub fn validator_name(&self) -> TokenStream2 {
    match self {
      ProtoType::String => quote! { StringValidator },
      ProtoType::Bool => quote! { BoolValidator },
      ProtoType::Bytes => quote! { BytesValidator },
      ProtoType::Enum(path) => quote! { EnumValidator<#path> },
      ProtoType::Message { path, .. } => quote! { MessageValidator<#path> },
      ProtoType::Int32 => quote! { IntValidator<i32> },
      ProtoType::Sint32 => quote! { IntValidator<::prelude::Sint32> },
      ProtoType::Duration => quote! { DurationValidator },
      ProtoType::Timestamp => quote! { TimestampValidator },
      ProtoType::Uint32 => quote! { IntValidator<u32> },
      ProtoType::Float => quote! { FloatValidator<f32> },
      ProtoType::Double => quote! { DoubleValidator<f64> },
      ProtoType::Int64 => quote! { IntValidator<i64> },
      ProtoType::Uint64 => quote! { IntValidator<u64> },
      ProtoType::Sint64 => quote! { IntValidator<prelude::Sint64>  },
      ProtoType::Fixed32 => quote! { IntValidator<prelude::Fixed32>  },
      ProtoType::Fixed64 => quote! { IntValidator<prelude::Fixed64>  },
      ProtoType::Sfixed32 => quote! { IntValidator<prelude::Sfixed32>  },
      ProtoType::Sfixed64 => quote! { IntValidator<prelude::Sfixed64>  },
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
      ProtoType::Uint32 => "uint32".into(),
      ProtoType::Float => "float".into(),
      ProtoType::Double => "double".into(),
      ProtoType::Int64 => "int64".into(),
      ProtoType::Uint64 => "uint64".into(),
      ProtoType::Sint64 => "sint64".into(),
      ProtoType::Fixed32 => "fixed32".into(),
      ProtoType::Fixed64 => "fixed64".into(),
      ProtoType::Sfixed32 => "sfixed32".into(),
      ProtoType::Sfixed64 => "sfixed64".into(),
    }
  }

  pub fn output_proto_type(&self) -> TokenStream2 {
    match self {
      ProtoType::String => quote! { String },
      ProtoType::Bool => quote! { bool },
      ProtoType::Bytes => quote! { ::bytes::Bytes },
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
      ProtoType::Uint32 => quote! { u32 },
      ProtoType::Float => quote! { f32 },
      ProtoType::Double => quote! { f64 },
      ProtoType::Int64 => quote! { i64  },
      ProtoType::Uint64 => quote! { u64 },
      ProtoType::Sint64 => quote! { i64  },
      ProtoType::Fixed32 => quote! { u32 },
      ProtoType::Fixed64 => quote! { u64 },
      ProtoType::Sfixed32 => quote! { i32 },
      ProtoType::Sfixed64 => quote! { i64 },
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
      ProtoType::Uint32 => quote! { uint32 },
      ProtoType::Float => quote! { float },
      ProtoType::Double => quote! { double },
      ProtoType::Int64 => quote! { int64  },
      ProtoType::Uint64 => quote! { uint64 },
      ProtoType::Sint64 => quote! { sint64  },
      ProtoType::Fixed32 => quote! { fixed32  },
      ProtoType::Fixed64 => quote! { fixed64  },
      ProtoType::Sfixed32 => quote! { sfixed32  },
      ProtoType::Sfixed64 => quote! { sfixed64  },
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
