#![allow(unused)]

use crate::*;
use ::proto_types::protovalidate::FieldRules;
use ::proto_types::protovalidate::Ignore;
use ::proto_types::protovalidate::MessageRules;
use ::proto_types::protovalidate::OneofRules;
use ::proto_types::protovalidate::Rule;
use ::proto_types::protovalidate::field_rules::Type as RulesType;
use proc_macro2::TokenTree;
use prost_reflect::prost::Message;
use prost_reflect::{Value as ProstValue, *};
mod pool_loader;
pub use pool_loader::*;
mod string_rules;
pub use string_rules::*;
mod numeric_rules;
pub use numeric_rules::*;
mod bool_rules;
pub use bool_rules::*;
mod bytes_rules;
pub use bytes_rules::*;
mod duration_rules;
pub use duration_rules::*;
mod timestamp_rules;
pub use timestamp_rules::*;
mod any_rules;
pub use any_rules::*;
mod field_mask_rules;
pub use field_mask_rules::*;

pub struct RulesCtx {
  pub ignore: IgnoreWrapper,
  pub cel: Vec<Rule>,
  pub required: bool,
}

impl RulesCtx {
  pub fn tokenize_cel_rules(&self, validator: &mut TokenStream2) {
    for rule in &self.cel {
      let Rule {
        id,
        message,
        expression,
      } = rule;

      validator.extend(quote! {
        .cel(::prelude::cel_program!(id = #id, msg = #message, expr = #expression))
      });
    }
  }

  pub fn tokenize_required(&self, validator: &mut TokenStream2) {
    if self.required {
      validator.extend(quote! { .required() });
    }
  }
}

pub struct IgnoreWrapper(Ignore);

impl IgnoreWrapper {
  pub fn tokenize(&self, validator: &mut TokenStream2) {
    match &self.0 {
      Ignore::Always => {
        validator.extend(quote! { .ignore_always() });
      }
      Ignore::IfZeroValue => {
        validator.extend(quote! { .ignore_if_zero_value() });
      }
      _ => {}
    };
  }

  pub fn tokenize_always_only(&self, validator: &mut TokenStream2) {
    if let Ignore::Always = &self.0 {
      validator.extend(quote! { .ignore_always() });
    }
  }
}

pub fn reflection_derive(item: &mut ItemStruct) -> Result<TokenStream2, Error> {
  let ItemStruct { fields, .. } = item;

  let mut msg_name: Option<String> = None;

  for attr in &item.attrs {
    if attr.path().is_ident("proto") {
      attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("name") {
          msg_name = Some(meta.parse_value::<LitStr>()?.value());
        } else {
          drain_token_stream!(meta.input);
        }

        Ok(())
      })?
    }
  }

  let msg_name = msg_name.ok_or_else(|| error_call_site!("Missing message name"))?;

  let message_desc = match DESCRIPTOR_POOL.get_message_by_name(&msg_name) {
    Some(message) => message,
    None => {
      bail_call_site!("Message {msg_name} not found in the descriptor pool");
    }
  };

  let mut fields_data: Vec<FieldData> = Vec::new();

  for field in fields {
    let ident = field.require_ident()?;
    let ident_str = ident.to_string();

    let mut oneof_path: Option<Path> = None;

    for attr in &field.attrs {
      if attr.path().is_ident("prost") {
        attr.parse_nested_meta(|meta| {
          if meta.path.is_ident("oneof") {
            let path_str = meta.parse_value::<LitStr>()?;

            oneof_path = Some(path_str.parse()?);
          }

          while !meta.input.is_empty() {
            meta.input.parse::<TokenTree>()?;
          }

          Ok(())
        })?;
      }
    }

    let proto_name = rust_ident_to_proto_name(&ident_str);

    if let Some(oneof_path) = oneof_path {
      let oneof = message_desc
        .oneofs()
        .find(|oneof| oneof.name() == proto_name)
        .ok_or_else(|| error!(field, "Oneof `{proto_name}` missing in the descriptor"))?;

      if let ProstValue::Message(oneof_rules_msg) = oneof
        .options()
        .get_extension(&ONEOF_RULES_EXT_DESCRIPTOR)
        .as_ref()
      {
        let oneof_rules = OneofRules::decode(oneof_rules_msg.encode_to_vec().as_slice())
          .map_err(|e| error!(field, "Could not decode oneof rules: {}", e))?;
      }
    } else {
      let proto = message_desc
        .get_field_by_name(proto_name)
        .ok_or_else(|| error!(field, "Field `{proto_name}` not found in the descriptor"))?;

      let type_info = TypeInfo::from_type(&field.ty)?;

      let proto_field = ProtoField::from_descriptor(&proto, &type_info);

      if let ProstValue::Message(field_rules_msg) = proto
        .options()
        .get_extension(&FIELD_RULES_EXT_DESCRIPTOR)
        .as_ref()
      {
        let field_rules = FieldRules::decode(field_rules_msg.encode_to_vec().as_slice())
          .map_err(|e| error!(field, "Could not decode field rules: {e}"))?;

        let ignore = field_rules.ignore();
        let is_required = field_rules.required() && proto.supports_presence();

        if matches!(ignore, Ignore::Always) {
          continue;
        }

        let rules_ctx = RulesCtx {
          ignore: IgnoreWrapper(ignore),
          required: field_rules.required(),
          cel: field_rules.cel,
        };

        let validator = if let Some(rules_type) = &field_rules.r#type {
          if proto.is_list() {
            todo!()
          } else if proto.is_map() {
            todo!()
          } else {
            let expr = match rules_type {
              RulesType::Float(rules) => get_numeric_validator(rules, &rules_ctx),
              RulesType::Double(rules) => get_numeric_validator(rules, &rules_ctx),
              RulesType::Int32(rules) => get_numeric_validator(rules, &rules_ctx),
              RulesType::Int64(rules) => get_numeric_validator(rules, &rules_ctx),
              RulesType::Uint32(rules) => get_numeric_validator(rules, &rules_ctx),
              RulesType::Uint64(rules) => get_numeric_validator(rules, &rules_ctx),
              RulesType::Sint32(rules) => get_numeric_validator(rules, &rules_ctx),
              RulesType::Sint64(rules) => get_numeric_validator(rules, &rules_ctx),
              RulesType::Fixed32(rules) => get_numeric_validator(rules, &rules_ctx),
              RulesType::Fixed64(rules) => get_numeric_validator(rules, &rules_ctx),
              RulesType::Sfixed32(rules) => get_numeric_validator(rules, &rules_ctx),
              RulesType::Sfixed64(rules) => get_numeric_validator(rules, &rules_ctx),
              RulesType::String(rules) => get_string_validator(rules, &rules_ctx),
              RulesType::Bool(rules) => get_bool_validator(rules, &rules_ctx),
              RulesType::Bytes(rules) => get_bytes_validator(rules, &rules_ctx),
              RulesType::Duration(rules) => get_duration_validator(rules, &rules_ctx),
              RulesType::Timestamp(rules) => get_timestamp_validator(rules, &rules_ctx),
              RulesType::Any(rules) => get_any_validator(rules, &rules_ctx),
              RulesType::FieldMask(rules) => get_field_mask_validator(rules, &rules_ctx),
              RulesType::Enum(rules) => todo!(),
              RulesType::Repeated(rules) => todo!(),
              RulesType::Map(rules) => todo!(),
            };

            ValidatorTokens {
              expr,
              is_fallback: false,
            }
          }
        } else {
          continue;
        };

        fields_data.push(FieldData {
          span: field.span(),
          ident: ident.clone(),
          type_info,
          proto_name: proto_name.to_string(),
          ident_str,
          tag: Some(proto.number().cast_signed()),
          validator: Some(validator),
          options: TokensOr::<TokenStream2>::new(|| quote! {}),
          proto_field,
          from_proto: None,
          into_proto: None,
        });
      }
    }
  }

  // Message Rules
  if let ProstValue::Message(message_rules_msg) = message_desc
    .options()
    .get_extension(&MESSAGE_RULES_EXT_DESCRIPTOR)
    .as_ref()
  {
    let message_rules = MessageRules::decode(message_rules_msg.encode_to_vec().as_slice())
      .map_err(|e| error!(item, "Could not decode message rules: {e}"))?;

    if !message_rules.cel.is_empty() {}
  }

  let validator_impl = generate_message_validator(
    &item.ident,
    &fields_data,
    &IterTokensOr::<TokenStream2>::vec(),
  );

  Ok(wrap_with_imports(vec![validator_impl]))
}

fn rust_ident_to_proto_name(rust_ident: &str) -> &str {
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
  pub fn from_descriptor(desc: &FieldDescriptor, type_info: &TypeInfo) -> Self {
    if desc.is_list() {
      Self::Repeated(ProtoType::from_descriptor(desc.kind(), type_info))
    } else if desc.is_map()
      && let Kind::Message(map_desc) = desc.kind()
    {
      let keys = ProtoMapKeys::from_descriptor(map_desc.map_entry_key_field().kind());
      let values = ProtoType::from_descriptor(map_desc.map_entry_value_field().kind(), type_info);

      Self::Map(ProtoMap { keys, values })
    } else if desc.supports_presence() {
      Self::Optional(ProtoType::from_descriptor(desc.kind(), type_info))
    } else {
      Self::Single(ProtoType::from_descriptor(desc.kind(), type_info))
    }
  }
}

impl ProtoType {
  #[allow(clippy::needless_pass_by_value)]
  pub fn from_descriptor(kind: Kind, type_info: &TypeInfo) -> Self {
    match kind {
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
        _ => Self::Message(MessageInfo {
          path: type_info.inner().as_path().unwrap(),
          boxed: type_info.is_box(),
        }),
      },
      Kind::Enum(_) => Self::Enum(type_info.inner().as_path().unwrap()),
    }
  }
}
