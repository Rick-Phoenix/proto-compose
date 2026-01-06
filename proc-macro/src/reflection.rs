use crate::*;
use ::proto_types::protovalidate::FieldRules;
use ::proto_types::protovalidate::Ignore;
use ::proto_types::protovalidate::MessageRules;
use ::proto_types::protovalidate::OneofRules;
use ::proto_types::protovalidate::Rule;
use ::proto_types::protovalidate::field_rules::Type as RulesType;
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
mod enum_rules;
pub use enum_rules::*;
mod message_rules;
pub use message_rules::*;
mod field_rules;
pub use field_rules::*;
mod repeated_rules;
pub use repeated_rules::*;
mod map_rules;
pub use map_rules::*;

pub struct RulesCtx<'a> {
  pub field_span: Span,
  pub rules: &'a FieldRules,
}

impl<'a> RulesCtx<'a> {
  pub fn from_non_empty_rules(rules: &'a FieldRules, field_span: Span) -> Option<Self> {
    if matches!(rules.ignore(), Ignore::Always)
      || (!rules.required() && rules.cel.is_empty() && rules.r#type.is_none())
    {
      None
    } else {
      Some(Self { field_span, rules })
    }
  }

  pub fn tokenize_cel_rules(&self, validator: &mut TokenStream2) {
    for rule in &self.rules.cel {
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
    if self.rules.required() {
      validator.extend(quote! { .required() });
    }
  }

  pub fn tokenize_ignore(&self, validator: &mut TokenStream2) {
    match self.rules.ignore() {
      Ignore::IfZeroValue => {
        validator.extend(quote! { .ignore_if_zero_value() });
      }
      _ => {}
    };
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

    let type_info = TypeInfo::from_type(&field.ty)?;
    let mut proto_field: Option<ProtoField> = None;
    let mut found_enum_path: Option<Path> = None;

    for attr in &field.attrs {
      if attr.path().is_ident("prost") {
        attr.parse_nested_meta(|meta| {
          let ident_str = meta.ident_str()?;

          match ident_str.as_str() {
            "map" => {
              let val = meta.parse_value::<LitStr>()?.value();
              let (_, value) = val
                .split_once(", ")
                .ok_or_else(|| meta.error("Failed to parse map attribute"))?;

              if let Some((_, wrapped_path)) = value.split_once("enumeration") {
                let str_path = &wrapped_path[1..wrapped_path.len() - 1];
                found_enum_path = Some(syn::parse_str(str_path)?);
              }
            }
            "oneof" => {
              let path_str = meta.parse_value::<LitStr>()?;

              proto_field = Some(ProtoField::Oneof(OneofInfo {
                path: path_str.parse()?,
                tags: vec![],
                default: false,
                required: false,
              }));
            }
            "enumeration" => {
              let path_str = meta.parse_value::<LitStr>()?;

              found_enum_path = Some(path_str.parse()?);
            }
            _ => drain_token_stream!(meta.input),
          };

          Ok(())
        })?;
      }
    }

    let proto_name = rust_ident_to_proto_name(&ident_str);

    if let Some(ProtoField::Oneof(mut oneof)) = proto_field {
      let oneof_desc = message_desc
        .oneofs()
        .find(|oneof| oneof.name() == proto_name)
        .ok_or_else(|| error!(field, "Oneof `{proto_name}` missing in the descriptor"))?;

      if let ProstValue::Message(oneof_rules_msg) = oneof_desc
        .options()
        .get_extension(&ONEOF_RULES_EXT_DESCRIPTOR)
        .as_ref()
      {
        let oneof_rules = OneofRules::decode(oneof_rules_msg.encode_to_vec().as_slice())
          .map_err(|e| error!(field, "Could not decode oneof rules: {}", e))?;

        oneof.required = oneof_rules.required();
      }
    } else {
      let field_desc = message_desc
        .get_field_by_name(proto_name)
        .ok_or_else(|| error!(field, "Field `{proto_name}` not found in the descriptor"))?;

      let proto_field = ProtoField::from_descriptor(&field_desc, &type_info, found_enum_path)?;

      let validator = if let ProstValue::Message(field_rules_msg) = field_desc
        .options()
        .get_extension(&FIELD_RULES_EXT_DESCRIPTOR)
        .as_ref()
        && let Some(rules_ctx) = RulesCtx::from_non_empty_rules(
          &FieldRules::decode(field_rules_msg.encode_to_vec().as_ref())
            .expect("Failed to decode field rules"),
          field.span(),
        ) {
        let expr = match &proto_field {
          ProtoField::Map(proto_map) => get_map_validator(&rules_ctx, proto_map),
          ProtoField::Oneof(_) => todo!(),
          ProtoField::Repeated(inner) => get_repeated_validator(&rules_ctx, inner),
          ProtoField::Optional(inner) | ProtoField::Single(inner) => {
            get_field_validator(&rules_ctx, inner).unwrap()
          }
        };

        ValidatorTokens {
          expr,
          is_fallback: false,
        }
      } else if let Some(fallback) = proto_field.default_validator_expr() {
        ValidatorTokens {
          expr: fallback,
          is_fallback: true,
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
        tag: Some(field_desc.number().cast_signed()),
        validator: Some(validator),
        options: TokensOr::<TokenStream2>::new(|| quote! {}),
        proto_field,
        from_proto: None,
        into_proto: None,
      });
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
      let keys = ProtoMapKeys::from_descriptor(map_desc.map_entry_key_field().kind());

      let RustType::HashMap((_, rust_values)) = type_info.type_.as_ref() else {
        bail!(type_info, "Found map descriptor on a non HashMap field");
      };

      let values = ProtoType::from_descriptor(
        map_desc.map_entry_value_field().kind(),
        rust_values,
        found_enum_path,
      )?;

      Self::Map(ProtoMap { keys, values })
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
