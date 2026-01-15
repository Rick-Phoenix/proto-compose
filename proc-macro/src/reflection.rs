use crate::*;
use ::proto_types::protovalidate::{
  FieldRules, Ignore, MessageRules, OneofRules, Rule, field_rules::Type as RulesType,
};
use prost_reflect::{Value as ProstValue, prost::Message, *};
mod pool_loader;
pub use pool_loader::*;
mod numeric_rules;
mod string_rules;
pub use numeric_rules::*;
mod any_rules;
mod bool_rules;
mod bytes_rules;
mod duration_rules;
mod enum_rules;
mod field_mask_rules;
mod field_rules;
mod map_rules;
mod message_rules;
mod oneof_derive;
mod repeated_rules;
mod timestamp_rules;
pub use oneof_derive::*;
mod message_derive;
pub use message_derive::*;

pub struct BuilderTokens {
  pub builder_expr: TokenStream2,
  pub methods_tokens: TokenStream2,
  pub span: Span,
}

impl BuilderTokens {
  pub fn new(span: Span, builder_expr: TokenStream2) -> Self {
    Self {
      builder_expr,
      methods_tokens: TokenStream2::new(),
      span,
    }
  }

  pub fn extend(&mut self, tokens: TokenStream2) {
    self.methods_tokens.extend(tokens);
  }

  pub fn into_builder(self) -> TokenStream2 {
    let Self {
      mut builder_expr,
      methods_tokens,
      ..
    } = self;

    methods_tokens.to_tokens(&mut builder_expr);

    quote! {
      ::prelude::#builder_expr
    }
  }

  pub fn into_built_validator(self) -> TokenStream2 {
    let Self {
      mut builder_expr,
      methods_tokens,
      span,
    } = self;

    methods_tokens.to_tokens(&mut builder_expr);

    quote_spanned! {span=>
      ::prelude::#builder_expr.build()
    }
  }
}

pub struct RulesCtx {
  pub field_span: Span,
  pub rules: FieldRules,
}

impl RulesCtx {
  pub fn from_descriptor(field_descriptor: &FieldDescriptor, field_span: Span) -> Option<Self> {
    if let ProstValue::Message(field_rules_msg) = field_descriptor
      .options()
      .get_extension(&FIELD_RULES_EXT_DESCRIPTOR)
      .as_ref()
    {
      let rules = FieldRules::decode(field_rules_msg.encode_to_vec().as_ref())
        .expect("Failed to decode field rules");

      Self::from_non_empty_rules(rules, field_span)
    } else {
      None
    }
  }

  pub fn from_non_empty_rules(rules: FieldRules, field_span: Span) -> Option<Self> {
    if matches!(rules.ignore(), Ignore::Always)
      || (!rules.required() && rules.cel.is_empty() && rules.r#type.is_none())
    {
      None
    } else {
      Some(Self { field_span, rules })
    }
  }

  pub fn tokenize_cel_rules(&self, validator: &mut BuilderTokens) {
    for rule in &self.rules.cel {
      let Rule {
        id,
        message,
        expression,
      } = rule;

      validator.extend(quote_spanned! {self.field_span=>
        .cel(::prelude::cel_program!(id = #id, msg = #message, expr = #expression))
      });
    }
  }

  pub fn tokenize_required(&self, validator: &mut BuilderTokens) {
    if self.rules.required() {
      validator.extend(quote_spanned! {self.field_span=> .required() });
    }
  }

  pub fn tokenize_ignore(&self, validator: &mut BuilderTokens) {
    match self.rules.ignore() {
      Ignore::IfZeroValue => {
        validator.extend(quote_spanned! {self.field_span=> .ignore_if_zero_value() });
      }
      _ => {}
    };
  }
}

pub fn rust_ident_to_proto_name(rust_ident: &str) -> &str {
  rust_ident
    .strip_prefix("r#")
    .unwrap_or(rust_ident.strip_suffix("_").unwrap_or(rust_ident))
}

impl ProtoMapKeys {
  #[allow(clippy::needless_pass_by_value)]
  pub fn from_descriptor(kind: Kind) -> Self {
    match kind {
      Kind::Int32 => Self::Int32,
      Kind::Int64 => Self::Int64,
      Kind::Uint32 => Self::Uint32,
      Kind::Uint64 => Self::Uint64,
      Kind::Sint32 => Self::Sint32,
      Kind::Sint64 => Self::Sint64,
      Kind::Fixed32 => Self::Fixed32,
      Kind::Fixed64 => Self::Fixed64,
      Kind::Sfixed32 => Self::Sfixed32,
      Kind::Sfixed64 => Self::Sfixed64,
      Kind::Bool => Self::Bool,
      Kind::String => Self::String,
      _ => unreachable!(),
    }
  }
}

impl ProtoField {
  pub fn from_descriptor(
    desc: &FieldDescriptor,
    type_info: &TypeInfo,
    found_enum_path: Option<Path>,
  ) -> syn::Result<Self> {
    let output = if desc.is_list() {
      let RustType::Vec(inner) = type_info.type_.as_ref() else {
        bail!(type_info, "Found repeated descriptor on a non Vec field");
      };

      Self::Repeated(ProtoType::from_descriptor(
        desc.kind(),
        inner,
        found_enum_path,
      )?)
    } else if desc.is_map()
      && let Kind::Message(map_desc) = desc.kind()
    {
      let mut is_btree_map = false;

      let keys = ProtoMapKeys::from_descriptor(map_desc.map_entry_key_field().kind());

      let rust_values = match type_info.type_.as_ref() {
        RustType::HashMap((_, v)) => v,
        RustType::BTreeMap((_, v)) => {
          is_btree_map = true;
          v
        }
        _ => bail!(type_info, "Found map descriptor on a non HashMap field"),
      };

      let values = ProtoType::from_descriptor(
        map_desc.map_entry_value_field().kind(),
        rust_values,
        found_enum_path,
      )?;

      Self::Map(ProtoMap {
        keys,
        values,
        is_btree_map,
      })
    } else if desc.supports_presence() {
      Self::Optional(ProtoType::from_descriptor(
        desc.kind(),
        type_info.inner(),
        found_enum_path,
      )?)
    } else {
      Self::Single(ProtoType::from_descriptor(
        desc.kind(),
        type_info,
        found_enum_path,
      )?)
    };

    Ok(output)
  }
}

impl ProtoType {
  #[allow(clippy::needless_pass_by_value)]
  pub fn from_descriptor(
    kind: Kind,
    type_info: &TypeInfo,
    found_enum_path: Option<Path>,
  ) -> syn::Result<Self> {
    let output = match kind {
      Kind::Double => Self::Double,
      Kind::Float => Self::Float,
      Kind::Int32 => Self::Int32,
      Kind::Int64 => Self::Int64,
      Kind::Uint32 => Self::Uint32,
      Kind::Uint64 => Self::Uint64,
      Kind::Sint32 => Self::Sint32,
      Kind::Sint64 => Self::Sint64,
      Kind::Fixed32 => Self::Fixed32,
      Kind::Fixed64 => Self::Fixed64,
      Kind::Sfixed32 => Self::Sfixed32,
      Kind::Sfixed64 => Self::Sfixed64,
      Kind::Bool => Self::Bool,
      Kind::String => Self::String,
      Kind::Bytes => Self::Bytes,
      Kind::Message(desc) => match desc.full_name() {
        "google.protobuf.Duration" => Self::Duration,
        "google.protobuf.Timestamp" => Self::Timestamp,
        "google.protobuf.Any" => Self::Any,
        "google.protobuf.FieldMask" => Self::FieldMask,
        _ => {
          let mut boxed = false;
          let inner = if type_info.is_box() {
            boxed = true;
            type_info.inner()
          } else {
            type_info
          };

          Self::Message(MessageInfo {
            path: inner
              .as_path()
              .ok_or_else(|| error!(type_info, "Failed to infer message path"))?,
            boxed,
            default: false,
          })
        }
      },
      Kind::Enum(_) => {
        Self::Enum(found_enum_path.ok_or_else(|| error!(type_info, "Failed to infer enum path"))?)
      }
    };

    Ok(output)
  }
}
