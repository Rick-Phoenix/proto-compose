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
  pub const fn is_message(&self) -> bool {
    matches!(
      self,
      Self::Message { .. } | Self::Duration | Self::Timestamp
    )
  }

  /// Returns `true` if the proto type is [`Enum`].
  ///
  /// [`Enum`]: ProtoType::Enum
  #[must_use]
  pub const fn is_enum(&self) -> bool {
    matches!(self, Self::Enum(..))
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

  pub fn from_nested_meta(
    ident_str: &str,
    meta: &ParseNestedMeta,
    type_info: Option<&TypeInfo>,
  ) -> Result<Self, Error> {
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
        let msg_info = MessageInfo::parse(meta, type_info)?;

        Self::Message(msg_info)
      }
      "bytes" => Self::Bytes,
      "bool" => Self::Bool,
      "duration" => Self::Duration,
      "timestamp" => Self::Timestamp,
      "enum_" => {
        let path = if let Ok(path) = meta.parse_list::<Path>() {
          path
        } else {
          type_info
            .and_then(|type_info| type_info.as_path())
            .ok_or_else(|| meta.error("Failed to infer the enum path. Please set it manually"))?
        };

        Self::Enum(path)
      }
      "any" => Self::Any,
      "field_mask" => Self::FieldMask,
      _ => return Err(meta_error!(meta, "Unknown protobuf type {ident_str}")),
    };

    Ok(output)
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

  pub fn field_proto_type_tokens(&self, span: Span) -> TokenStream2 {
    match self {
      Self::String => quote_spanned! {span=> String },
      Self::Bool => quote_spanned! {span=> bool },
      Self::Bytes => quote_spanned! {span=> ::bytes::Bytes },
      Self::Enum(path) | Self::Message(MessageInfo { path, .. }) => path.to_token_stream(),
      Self::Int32 => quote_spanned! {span=> i32 },
      Self::Sint32 => quote_spanned! {span=> ::prelude::Sint32 },
      Self::Duration => quote_spanned! {span=> ::prelude::proto_types::Duration },
      Self::Timestamp => quote_spanned! {span=> ::prelude::proto_types::Timestamp },
      Self::Uint32 => quote_spanned! {span=> u32 },
      Self::Float => quote_spanned! {span=> f32 },
      Self::Double => quote_spanned! {span=> f64 },
      Self::Int64 => quote_spanned! {span=> i64  },
      Self::Uint64 => quote_spanned! {span=> u64 },
      Self::Sint64 => quote_spanned! {span=> ::prelude::Sint64  },
      Self::Fixed32 => quote_spanned! {span=> ::prelude::Fixed32  },
      Self::Fixed64 => quote_spanned! {span=> ::prelude::Fixed64  },
      Self::Sfixed32 => quote_spanned! {span=> ::prelude::Sfixed32  },
      Self::Sfixed64 => quote_spanned! {span=> ::prelude::Sfixed64  },
      Self::Any => quote_spanned! {span=> ::prelude::proto_types::Any },
      Self::FieldMask => quote_spanned! {span=> ::prelude::proto_types::FieldMask },
    }
  }

  pub fn descriptor_type_tokens(&self, span: Span) -> TokenStream2 {
    let prefix = quote_spanned! {span=> ::prelude::proto_types::field_descriptor_proto::Type };

    match self {
      Self::Uint32 => quote_spanned! {span=> #prefix::Uint32 },
      Self::String => quote_spanned! {span=> #prefix::String },
      Self::Bool => quote_spanned! {span=> #prefix::Bool },
      Self::Bytes => quote_spanned! {span=> #prefix::Bytes },
      Self::Enum(_) => quote_spanned! {span=> #prefix::Enum },
      Self::Int32 => quote_spanned! {span=> #prefix::Int32 },
      Self::Sint32 => quote_spanned! {span=> #prefix::Sint32 },
      Self::Float => quote_spanned! {span=> #prefix::Float },
      Self::Double => quote_spanned! {span=> #prefix::Double },
      Self::Int64 => quote_spanned! {span=> #prefix::Int64  },
      Self::Uint64 => quote_spanned! {span=> #prefix::Uint64  },
      Self::Sint64 => quote_spanned! {span=> #prefix::Sint64  },
      Self::Fixed32 => quote_spanned! {span=> #prefix::Fixed32  },
      Self::Fixed64 => quote_spanned! {span=> #prefix::Fixed64  },
      Self::Sfixed32 => quote_spanned! {span=> #prefix::Sfixed32  },
      Self::Sfixed64 => quote_spanned! {span=> #prefix::Sfixed64  },
      _ => quote_spanned! {span=> #prefix::Message },
    }
  }

  pub fn default_from_proto(&self, base_ident: &TokenStream2) -> TokenStream2 {
    let span = base_ident.span();

    match self {
      Self::Enum(_) => quote_spanned! {span=> #base_ident.try_into().unwrap_or_default() },
      Self::Message(MessageInfo {
        boxed,
        default,
        path,
        ..
      }) => {
        if *default {
          if *boxed {
            quote_spanned! {span=> #base_ident.map(|v| Box::new((*v).into())).unwrap_or_else(|| Box::new(#path::default().into())) }
          } else {
            quote_spanned! {span=> #base_ident.map(|v| v.into()).unwrap_or_else(|| #path::default().into()) }
          }
        } else {
          if *boxed {
            quote_spanned! {span=> Box::new((*#base_ident).into()) }
          } else {
            quote_spanned! {span=> #base_ident.into() }
          }
        }
      }
      _ => quote_spanned! {span=> #base_ident.into() },
    }
  }

  pub fn validator_target_type(&self, span: Span) -> TokenStream2 {
    match self {
      Self::String => quote_spanned! {span=> String },
      Self::Bool => quote_spanned! {span=> bool },
      Self::Bytes => quote_spanned! {span=> ::bytes::Bytes },
      Self::Enum(path) | Self::Message(MessageInfo { path, .. }) => path.to_token_stream(),
      Self::Int32 => quote_spanned! {span=> i32 },
      Self::Sint32 => quote_spanned! {span=> ::prelude::Sint32 },
      Self::Duration => quote_spanned! {span=> ::prelude::proto_types::Duration },
      Self::Timestamp => quote_spanned! {span=> ::prelude::proto_types::Timestamp },
      Self::Uint32 => quote_spanned! {span=> u32 },
      Self::Float => quote_spanned! {span=> f32 },
      Self::Double => quote_spanned! {span=> f64 },
      Self::Int64 => quote_spanned! {span=> i64  },
      Self::Uint64 => quote_spanned! {span=> u64 },
      Self::Sint64 => quote_spanned! {span=> ::prelude::Sint64  },
      Self::Fixed32 => quote_spanned! {span=> ::prelude::Fixed32  },
      Self::Fixed64 => quote_spanned! {span=> ::prelude::Fixed64  },
      Self::Sfixed32 => quote_spanned! {span=> ::prelude::Sfixed32  },
      Self::Sfixed64 => quote_spanned! {span=> ::prelude::Sfixed64  },
      Self::Any => quote_spanned! {span=> ::prelude::proto_types::Any },
      Self::FieldMask => quote_spanned! {span=> ::prelude::proto_types::FieldMask },
    }
  }

  pub fn validator_name(&self, span: Span) -> TokenStream2 {
    match self {
      Self::String => quote_spanned! {span=> ::prelude::StringValidator },
      Self::Bool => quote_spanned! {span=> ::prelude::BoolValidator },
      Self::Bytes => quote_spanned! {span=> ::prelude::BytesValidator },
      Self::Enum(path) => quote_spanned! {span=> ::prelude::EnumValidator<#path> },
      Self::Message(MessageInfo { path, .. }) => {
        quote_spanned! {span=> ::prelude::MessageValidator<#path> }
      }
      Self::Int32 => quote_spanned! {span=> ::prelude::IntValidator<i32> },
      Self::Sint32 => quote_spanned! {span=> ::prelude::IntValidator<::prelude::Sint32> },
      Self::Duration => quote_spanned! {span=> ::prelude::DurationValidator },
      Self::Timestamp => quote_spanned! {span=> ::prelude::TimestampValidator },
      Self::Uint32 => quote_spanned! {span=> ::prelude::IntValidator<u32> },
      Self::Float => quote_spanned! {span=> ::prelude::FloatValidator<f32> },
      Self::Double => quote_spanned! {span=> ::prelude::FloatValidator<f64> },
      Self::Int64 => quote_spanned! {span=> ::prelude::IntValidator<i64> },
      Self::Uint64 => quote_spanned! {span=> ::prelude::IntValidator<u64> },
      Self::Sint64 => quote_spanned! {span=> ::prelude::IntValidator<prelude::Sint64>  },
      Self::Fixed32 => quote_spanned! {span=> ::prelude::IntValidator<prelude::Fixed32>  },
      Self::Fixed64 => quote_spanned! {span=> ::prelude::IntValidator<prelude::Fixed64>  },
      Self::Sfixed32 => quote_spanned! {span=> ::prelude::IntValidator<prelude::Sfixed32>  },
      Self::Sfixed64 => quote_spanned! {span=> ::prelude::IntValidator<prelude::Sfixed64>  },
      Self::Any => quote_spanned! {span=> ::prelude::AnyValidator },
      Self::FieldMask => quote_spanned! {span=> ::prelude::FieldMaskValidator },
    }
  }

  pub fn output_proto_type(&self, span: Span) -> TokenStream2 {
    match self {
      Self::String => quote_spanned! {span=> String },
      Self::Bool => quote_spanned! {span=> bool },
      Self::Bytes => quote_spanned! {span=> ::bytes::Bytes },
      Self::Enum(_) | Self::Int32 | Self::Sint32 | Self::Sfixed32 => quote_spanned! {span=> i32 },
      Self::Message(MessageInfo { boxed, path, .. }) => {
        if *boxed {
          quote_spanned! {span=> Box<#path> }
        } else {
          path.to_token_stream()
        }
      }
      Self::Duration => quote_spanned! {span=> ::prelude::proto_types::Duration },
      Self::Timestamp => quote_spanned! {span=> ::prelude::proto_types::Timestamp },
      Self::Any => quote_spanned! {span=> ::prelude::proto_types::Any },
      Self::FieldMask => quote_spanned! {span=> ::prelude::proto_types::FieldMask },
      Self::Uint32 | Self::Fixed32 => quote_spanned! {span=> u32 },
      Self::Float => quote_spanned! {span=> f32 },
      Self::Double => quote_spanned! {span=> f64 },
      Self::Int64 | Self::Sint64 | Self::Sfixed64 => quote_spanned! {span=> i64  },
      Self::Uint64 | Self::Fixed64 => quote_spanned! {span=> u64 },
    }
  }

  pub fn as_prost_attr_type(&self, span: Span) -> TokenStream2 {
    match self {
      Self::String => quote_spanned! {span=> string },
      Self::Bool => quote_spanned! {span=> bool },
      Self::Bytes => quote_spanned! {span=> bytes = "bytes" },
      Self::Enum(path) => {
        let path_as_str = path.to_token_stream().to_string();

        quote_spanned! {span=> enumeration = #path_as_str }
      }
      Self::Message(MessageInfo { boxed, .. }) => {
        if *boxed {
          quote_spanned! {span=> message, boxed }
        } else {
          quote_spanned! {span=> message }
        }
      }
      Self::Int32 => quote_spanned! {span=> int32 },
      Self::Sint32 => quote_spanned! {span=> sint32 },
      Self::Uint32 => quote_spanned! {span=> uint32 },
      Self::Float => quote_spanned! {span=> float },
      Self::Double => quote_spanned! {span=> double },
      Self::Int64 => quote_spanned! {span=> int64  },
      Self::Uint64 => quote_spanned! {span=> uint64 },
      Self::Sint64 => quote_spanned! {span=> sint64  },
      Self::Fixed32 => quote_spanned! {span=> fixed32  },
      Self::Fixed64 => quote_spanned! {span=> fixed64  },
      Self::Sfixed32 => quote_spanned! {span=> sfixed32  },
      Self::Sfixed64 => quote_spanned! {span=> sfixed64  },
      _ => quote_spanned! {span=> message },
    }
  }
}
