use ::bytes::Bytes;
use bon::Builder;
use bytes_validator_builder::{IsUnset, SetWellKnown, State};
use regex::Regex;

use super::*;

impl_validator!(BytesValidator, Bytes);
impl_into_option!(BytesValidator);
impl_ignore!(BytesValidatorBuilder);

macro_rules! insert_bytes_option {
  ($validator:ident, $values:ident, $field:ident) => {
    $validator.$field.map(|v| {
      $values.push((
        $crate::paste!([< $field:upper >]).clone(),
        OptionValue::String(format_bytes_as_proto_string_literal(&v).into()),
      ))
    })
  };

  ($validator:ident, $values:ident, $field:ident, list) => {
    $validator.$field.map(|v| {
      $values.push((
        $crate::paste!([< $field:upper >]).clone(),
        OptionValue::List(
          v.iter()
            .map(|i| OptionValue::String(format_bytes_as_proto_string_literal(i).into()))
            .collect::<Vec<OptionValue>>()
            .into(),
        ),
      ))
    })
  };
}

#[derive(Clone, Debug, Builder)]
#[builder(derive(Clone))]
pub struct BytesValidator {
  /// Specifies that the given `bytes` field must be of this exact length.
  pub len: Option<u64>,
  /// Specifies that the given `bytes` field must have a length that is equal to or higher than the given value.
  pub min_len: Option<u64>,
  /// Specifies that the given `bytes` field must have a length that is equal to or lower than the given value.
  pub max_len: Option<u64>,
  /// Specifies a regex pattern that must be matches by the value to pass validation.
  pub pattern: Option<Regex>,
  /// Specifies a prefix that the value must start with in order to pass validation.
  pub prefix: Option<Bytes>,
  /// Specifies a suffix that the value must end with in order to pass validation.
  pub suffix: Option<Bytes>,
  /// Specifies a subset of bytes that the value must contain in order to pass validation.
  pub contains: Option<Bytes>,
  /// Specifies that only the values in this list will be considered valid for this field.
  #[builder(into)]
  pub in_: Option<Arc<[Bytes]>>,
  /// Specifies that the values in this list will be considered NOT valid for this field.
  #[builder(into)]
  pub not_in: Option<Arc<[Bytes]>>,
  #[builder(setters(vis = "", name = well_known))]
  pub well_known: Option<WellKnownBytes>,
  /// Specifies that only this specific value will be considered valid for this field.
  pub const_: Option<Bytes>,
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  #[builder(into)]
  pub cel: Option<Arc<[CelRule]>>,
  #[builder(with = || true)]
  /// Specifies that the field must be set in order to be valid.
  pub required: Option<bool>,
  #[builder(setters(vis = "", name = ignore))]
  pub ignore: Option<Ignore>,
}

impl From<BytesValidator> for ProtoOption {
  fn from(validator: BytesValidator) -> Self {
    let mut rules: OptionValueList = Vec::new();

    if let Some(const_val) = validator.const_ {
      rules.push((
        CONST_.clone(),
        OptionValue::String(format_bytes_as_proto_string_literal(&const_val).into()),
      ));
    }

    if validator.len.is_none() {
      insert_option!(validator, rules, min_len);
      insert_option!(validator, rules, max_len);
    } else {
      insert_option!(validator, rules, len);
    }

    if let Some(pattern) = validator.pattern {
      rules.push((
        PATTERN.clone(),
        OptionValue::String(pattern.as_str().into()),
      ))
    }

    insert_bytes_option!(validator, rules, contains);
    insert_bytes_option!(validator, rules, prefix);
    insert_bytes_option!(validator, rules, suffix);
    insert_bytes_option!(validator, rules, in_, list);
    insert_bytes_option!(validator, rules, not_in, list);

    if let Some(v) = validator.well_known {
      v.to_option(&mut rules);
    }

    let mut outer_rules: OptionValueList = vec![];

    outer_rules.push((BYTES.clone(), OptionValue::Message(rules.into())));

    insert_cel_rules!(validator, outer_rules);
    insert_option!(validator, outer_rules, required);
    insert_option!(validator, outer_rules, ignore);

    ProtoOption {
      name: BUF_VALIDATE_FIELD.clone(),
      value: OptionValue::Message(outer_rules.into()),
    }
  }
}

#[doc(hidden)]
#[derive(Clone, Debug, Copy)]
pub enum WellKnownBytes {
  Ip,
  Ipv4,
  Ipv6,
}

macro_rules! well_known_impl {
  ($name:ident, $doc:literal) => {
    paste::paste! {
      #[doc = $doc]
      pub fn [< $name:snake >](self) -> BytesValidatorBuilder<SetWellKnown<S>>
        where
          S::WellKnown: IsUnset,
        {
          self.well_known(WellKnownBytes::$name)
        }
    }
  };
}

impl<S: State> BytesValidatorBuilder<S> {
  well_known_impl!(
    Ip,
    "Specifies that the value must be a valid IP address (v4 or v6) in byte format."
  );
  well_known_impl!(
    Ipv4,
    "Specifies that the value must be a valid IPv4 address in byte format."
  );
  well_known_impl!(
    Ipv6,
    "Specifies that the value must be a valid IPv6 address in byte format."
  );
}

impl WellKnownBytes {
  pub(crate) fn to_option(self, option_values: &mut OptionValueList) {
    let name = match self {
      WellKnownBytes::Ip => IP.clone(),
      WellKnownBytes::Ipv4 => IPV4.clone(),
      WellKnownBytes::Ipv6 => IPV6.clone(),
    };

    option_values.push((name, OptionValue::Bool(true)));
  }
}

fn format_bytes_as_proto_string_literal(bytes: &[u8]) -> String {
  let mut result = String::new();

  for &byte in bytes {
    match byte {
      0x20..=0x21 | 0x23..=0x5B | 0x5D..=0x7E => {
        result.push(byte as char);
      }
      b'\\' => result.push_str("\\\\"),
      b'"' => result.push_str("\\\""),
      _ => {
        result.push_str(&format!("\\x{:02x}", byte));
      }
    }
  }

  result
}
