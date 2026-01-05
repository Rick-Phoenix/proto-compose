use syn::spanned::Spanned;

use crate::*;

#[derive(Debug, Clone, Default)]
pub enum ProtoType {
  #[default]
  String,
  Bool,
  Bytes,
  Enum(Path),
  Message(MessageInfo),
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
  Any,
  FieldMask,
}

impl ProtoType {
  pub const fn is_custom_message(&self) -> bool {
    matches!(self, Self::Message { .. })
  }

  pub const fn is_message(&self) -> bool {
    matches!(
      self,
      Self::Message { .. } | Self::Duration | Self::Timestamp
    )
  }

  pub fn is_boxed_message(&self) -> bool {
    matches!(self, Self::Message(MessageInfo { boxed: true, .. }))
  }

  pub fn descriptor_type_tokens(&self) -> TokenStream2 {
    let prefix = quote! { ::prelude::proto_types::field_descriptor_proto::Type };

    match self {
      Self::Uint32 => quote! { #prefix::Uint32 },
      Self::String => quote! { #prefix::String },
      Self::Bool => quote! { #prefix::Bool },
      Self::Bytes => quote! { #prefix::Bytes },
      Self::Enum(_) => quote! { #prefix::Enum },
      Self::Int32 => quote! { #prefix::Int32 },
      Self::Sint32 => quote! { #prefix::Sint32 },
      Self::Float => quote! { #prefix::Float },
      Self::Double => quote! { #prefix::Double },
      Self::Int64 => quote! { #prefix::Int64  },
      Self::Uint64 => quote! { #prefix::Uint64  },
      Self::Sint64 => quote! { #prefix::Sint64  },
      Self::Fixed32 => quote! { #prefix::Fixed32  },
      Self::Fixed64 => quote! { #prefix::Fixed64  },
      Self::Sfixed32 => quote! { #prefix::Sfixed32  },
      Self::Sfixed64 => quote! { #prefix::Sfixed64  },
      _ => quote! { #prefix::Message },
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
        let msg_info = MessageInfo::parse(&meta, fallback)?;

        Self::Message(msg_info)
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

        Self::Message(MessageInfo { path, boxed: false })
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
      "any" => Self::Any,
      "field_mask" => Self::FieldMask,
      _ => return Err(error_with_span!(span, "Unknown protobuf type {ident_str}")),
    };

    Ok(output)
  }

  pub fn field_proto_type_tokens(&self) -> TokenStream2 {
    match self {
      Self::String => quote! { String },
      Self::Bool => quote! { bool },
      Self::Bytes => quote! { ::bytes::Bytes },
      Self::Enum(path) | Self::Message(MessageInfo { path, .. }) => quote! { #path },
      Self::Int32 => quote! { i32 },
      Self::Sint32 => quote! { prelude::Sint32 },
      Self::Duration => quote! { ::prelude::proto_types::Duration },
      Self::Timestamp => quote! { ::prelude::proto_types::Timestamp },
      Self::Uint32 => quote! { u32 },
      Self::Float => quote! { f32 },
      Self::Double => quote! { f64 },
      Self::Int64 => quote! { i64  },
      Self::Uint64 => quote! { u64 },
      Self::Sint64 => quote! { prelude::Sint64  },
      Self::Fixed32 => quote! { prelude::Fixed32  },
      Self::Fixed64 => quote! { prelude::Fixed64  },
      Self::Sfixed32 => quote! { prelude::Sfixed32  },
      Self::Sfixed64 => quote! { prelude::Sfixed64  },
      Self::Any => quote! { ::prelude::proto_types::Any },
      Self::FieldMask => quote! { ::prelude::proto_types::FieldMask },
    }
  }

  pub fn from_primitive(path: &Path) -> Result<Self, Error> {
    let ident = &path.segments.last().unwrap().ident;
    let ident_str = ident.to_string();

    let output = match ident_str.as_str() {
      "Bytes" => Self::Bytes,
      "String" => Self::String,
      "bool" => Self::Bool,
      "i32" => Self::Int32,
      "i64" => Self::Int64,
      "u32" => Self::Uint32,
      "u64" => Self::Uint64,
      "f32" => Self::Float,
      "f64" => Self::Double,
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
      Self::Enum(_) => quote! { #base_ident.try_into().unwrap_or_default() },
      Self::Message(MessageInfo { boxed: true, .. }) => {
        quote! { Box::new((*#base_ident).into()) }
      }
      _ => quote! { #base_ident.into() },
    }
  }

  pub fn default_into_proto(&self, base_ident: &TokenStream2) -> TokenStream2 {
    match self {
      Self::Message(MessageInfo { boxed: true, .. }) => {
        quote! { Box::new((*#base_ident).into()) }
      }
      _ => quote! { #base_ident.into() },
    }
  }

  pub fn validator_target_type(&self) -> TokenStream2 {
    match self {
      Self::String => quote! { String },
      Self::Bool => quote! { bool },
      Self::Bytes => quote! { ::bytes::Bytes },
      Self::Enum(path) | Self::Message(MessageInfo { path, .. }) => quote! { #path },
      Self::Int32 => quote! { i32 },
      Self::Sint32 => quote! { ::prelude::Sint32 },
      Self::Duration => quote! { ::prelude::proto_types::Duration },
      Self::Timestamp => quote! { ::prelude::proto_types::Timestamp },
      Self::Uint32 => quote! { u32 },
      Self::Float => quote! { f32 },
      Self::Double => quote! { f64 },
      Self::Int64 => quote! { i64  },
      Self::Uint64 => quote! { u64 },
      Self::Sint64 => quote! { prelude::Sint64  },
      Self::Fixed32 => quote! { prelude::Fixed32  },
      Self::Fixed64 => quote! { prelude::Fixed64  },
      Self::Sfixed32 => quote! { prelude::Sfixed32  },
      Self::Sfixed64 => quote! { prelude::Sfixed64  },
      Self::Any => quote! { ::prelude::proto_types::Any },
      Self::FieldMask => quote! { ::prelude::proto_types::FieldMask },
    }
  }

  pub fn validator_name(&self) -> TokenStream2 {
    match self {
      Self::String => quote! { StringValidator },
      Self::Bool => quote! { BoolValidator },
      Self::Bytes => quote! { BytesValidator },
      Self::Enum(path) => quote! { EnumValidator<#path> },
      Self::Message(MessageInfo { path, .. }) => quote! { MessageValidator<#path> },
      Self::Int32 => quote! { IntValidator<i32> },
      Self::Sint32 => quote! { IntValidator<::prelude::Sint32> },
      Self::Duration => quote! { DurationValidator },
      Self::Timestamp => quote! { TimestampValidator },
      Self::Uint32 => quote! { IntValidator<u32> },
      Self::Float => quote! { FloatValidator<f32> },
      Self::Double => quote! { FloatValidator<f64> },
      Self::Int64 => quote! { IntValidator<i64> },
      Self::Uint64 => quote! { IntValidator<u64> },
      Self::Sint64 => quote! { IntValidator<prelude::Sint64>  },
      Self::Fixed32 => quote! { IntValidator<prelude::Fixed32>  },
      Self::Fixed64 => quote! { IntValidator<prelude::Fixed64>  },
      Self::Sfixed32 => quote! { IntValidator<prelude::Sfixed32>  },
      Self::Sfixed64 => quote! { IntValidator<prelude::Sfixed64>  },
      Self::Any => quote! { AnyValidator },
      Self::FieldMask => quote! { FieldMaskValidator },
    }
  }

  pub fn as_prost_map_value(&self) -> Cow<'static, str> {
    match self {
      Self::String => "string".into(),
      Self::Bool => "bool".into(),
      Self::Bytes => "bytes".into(),
      Self::Enum(path) => {
        let path_as_str = path.to_token_stream().to_string();

        format!("enumeration({path_as_str})").into()
      }
      Self::Int32 => "int32".into(),
      Self::Sint32 => "sint32".into(),
      Self::Uint32 => "uint32".into(),
      Self::Float => "float".into(),
      Self::Double => "double".into(),
      Self::Int64 => "int64".into(),
      Self::Uint64 => "uint64".into(),
      Self::Sint64 => "sint64".into(),
      Self::Fixed32 => "fixed32".into(),
      Self::Fixed64 => "fixed64".into(),
      Self::Sfixed32 => "sfixed32".into(),
      Self::Sfixed64 => "sfixed64".into(),
      _ => "message".into(),
    }
  }

  pub fn output_proto_type(&self) -> TokenStream2 {
    match self {
      Self::String => quote! { String },
      Self::Bool => quote! { bool },
      Self::Bytes => quote! { Bytes },
      Self::Enum(_) | Self::Int32 | Self::Sint32 | Self::Sfixed32 => quote! { i32 },
      Self::Message(MessageInfo { boxed, path }) => {
        if *boxed {
          quote! { Box<#path> }
        } else {
          path.to_token_stream()
        }
      }
      Self::Duration => quote! { ::prelude::proto_types::Duration },
      Self::Timestamp => quote! { ::prelude::proto_types::Timestamp },
      Self::Any => quote! { ::prelude::proto_types::Any },
      Self::FieldMask => quote! { ::prelude::proto_types::FieldMask },
      Self::Uint32 | Self::Fixed32 => quote! { u32 },
      Self::Float => quote! { f32 },
      Self::Double => quote! { f64 },
      Self::Int64 | Self::Sint64 | Self::Sfixed64 => quote! { i64  },
      Self::Uint64 | Self::Fixed64 => quote! { u64 },
    }
  }

  pub fn as_prost_attr_type(&self) -> TokenStream2 {
    match self {
      Self::String => quote! { string },
      Self::Bool => quote! { bool },
      Self::Bytes => quote! { bytes = "bytes" },
      Self::Enum(path) => {
        let path_as_str = path.to_token_stream().to_string();

        quote! { enumeration = #path_as_str }
      }
      Self::Message(MessageInfo { boxed, .. }) => {
        if *boxed {
          quote! { message, boxed }
        } else {
          quote! { message }
        }
      }
      Self::Int32 => quote! { int32 },
      Self::Sint32 => quote! { sint32 },
      Self::Uint32 => quote! { uint32 },
      Self::Float => quote! { float },
      Self::Double => quote! { double },
      Self::Int64 => quote! { int64  },
      Self::Uint64 => quote! { uint64 },
      Self::Sint64 => quote! { sint64  },
      Self::Fixed32 => quote! { fixed32  },
      Self::Fixed64 => quote! { fixed64  },
      Self::Sfixed32 => quote! { sfixed32  },
      Self::Sfixed64 => quote! { sfixed64  },
      _ => quote! { message },
    }
  }
}
