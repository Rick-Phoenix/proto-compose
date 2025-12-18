use ::bytes::Bytes;
use bon::Builder;
use bytes_validator_builder::{IsUnset, SetWellKnown, State};
#[cfg(feature = "regex")]
use regex::bytes::Regex;

use super::*;

impl_validator!(BytesValidator, Bytes);
impl_into_option!(BytesValidator);
impl_ignore!(BytesValidatorBuilder);
impl_proto_type!(Bytes, "bytes");

pub(crate) fn format_bytes_list<'a, I: IntoIterator<Item = &'a [u8]>>(list: I) -> String {
  let mut string = String::new();
  let mut iter = list.into_iter().peekable();

  while let Some(item) = iter.next() {
    write!(string, "{}", item.escape_ascii()).unwrap();

    if iter.peek().is_some() {
      string.push_str(", ");
    }
  }

  string
}

impl Validator<Bytes> for BytesValidator {
  type Target = Bytes;

  #[cfg(feature = "testing")]
  fn cel_rules(&self) -> Vec<&'static CelRule> {
    self.cel.iter().map(|prog| &prog.rule).collect()
  }

  #[cfg(feature = "testing")]
  fn check_cel_programs_with(&self, val: Self::Target) -> Result<(), Vec<CelError>> {
    if !self.cel.is_empty() {
      // This one needs a special impl because Bytes does not support Into<Value>
      test_programs(&self.cel, val.to_vec())
    } else {
      Ok(())
    }
  }

  fn validate(
    &self,
    field_context: &FieldContext,
    parent_elements: &mut Vec<FieldPathElement>,
    val: Option<&Self::Target>,
  ) -> Result<(), Violations> {
    handle_ignore_always!(&self.ignore);
    handle_ignore_if_zero_value!(&self.ignore, val.is_none_or(|v| v.is_default()));

    let mut violations_agg = Violations::new();
    let violations = &mut violations_agg;

    if let Some(val) = val {
      if let Some(const_val) = &self.const_ && *val != const_val {
        violations.add(field_context, parent_elements, &BYTES_CONST_VIOLATION, &format!("must be equal to {}", const_val.escape_ascii()));
      }

      if let Some(len) = self.len && val.len() != len {
        violations.add(field_context, parent_elements, &BYTES_LEN_VIOLATION, &format!("must be exactly {len} bytes long"));
      }

      if let Some(min_len) = self.min_len && val.len() < min_len {
        violations.add(field_context, parent_elements, &BYTES_MIN_LEN_VIOLATION, &format!("must be at least {min_len} bytes long"));
      }

      if let Some(max_len) = self.max_len && val.len() > max_len {
        violations.add(field_context, parent_elements, &BYTES_MAX_LEN_VIOLATION, &format!("cannot be longer than {max_len} bytes"));
      }

      #[cfg(feature = "regex")]
      if let Some(pattern) = &self.pattern && !pattern.is_match(val) {
        violations.add(field_context, parent_elements, &BYTES_PATTERN_VIOLATION, &format!("must match the pattern `{pattern}`"));
      }

      if let Some(prefix) = &self.prefix && !val.starts_with(prefix) {
        violations.add(field_context, parent_elements, &BYTES_PREFIX_VIOLATION, &format!("must start with {}", prefix.escape_ascii()));
      }

      if let Some(suffix) = &self.suffix && !val.ends_with(suffix) {
        violations.add(field_context, parent_elements, &BYTES_SUFFIX_VIOLATION, &format!("must end with {}", suffix.escape_ascii()));
      }

      if let Some(substring) = &self.contains && !val.windows(val.len()).any(|slice| slice == substring) {
        violations.add(field_context, parent_elements, &BYTES_CONTAINS_VIOLATION, &format!("must contain {}", substring.escape_ascii()));
      }

      if let Some(allowed_list) = &self.in_ && !<&Bytes>::is_in(allowed_list, val) {
        violations.add(field_context, parent_elements, &BYTES_IN_VIOLATION, &format!("must be one of these values: {}", format_bytes_list(allowed_list.into_iter().copied())));
      }

      if let Some(forbidden_list) = &self.not_in && <&Bytes>::is_in(forbidden_list, val) {
        violations.add(field_context, parent_elements, &BYTES_IN_VIOLATION, &format!("cannot be one of these values: {}", format_bytes_list(forbidden_list.into_iter().copied())));
      }

      if let Some(well_known) = &self.well_known {
        let byte_str = core::str::from_utf8(val.as_ref()).unwrap_or("");

        match well_known {
          WellKnownBytes::Ip => {
            if !is_valid_ip(byte_str) {
              violations.add(
                field_context,
                parent_elements,
                &BYTES_IP_VIOLATION,
                "must be a valid ip address",
              );
            }
          }
          WellKnownBytes::Ipv4 => {
            if !is_valid_ipv4(byte_str) {
              violations.add(
                field_context,
                parent_elements,
                &BYTES_IPV4_VIOLATION,
                "must be a valid ipv4 address",
              );
            }
          }
          WellKnownBytes::Ipv6 => {
            if !is_valid_ipv6(byte_str) {
              violations.add(
                field_context,
                parent_elements,
                &BYTES_IPV6_VIOLATION,
                "must be a valid ipv6 address",
              );
            }
          }
        };
      }

      if !self.cel.is_empty() {
        let ctx = ProgramsExecutionCtx {
          programs: &self.cel,
          value: val.to_vec(),
          violations,
          field_context: Some(field_context),
          parent_elements,
        };

        ctx.execute_programs();
      }
    } else if self.required {
      violations.add_required(field_context, parent_elements);
    }

    if violations.is_empty() {
      Ok(())
    } else {
      Err(violations_agg)
    }
  }
}

macro_rules! insert_bytes_option {
  ($validator:ident, $values:ident, $field:ident) => {
    $validator.$field.map(|v| {
      $values.push((
        $crate::paste!([< $field:upper >]).clone(),
        OptionValue::String(format_bytes_as_proto_string_literal(&v).into()),
      ))
    })
  };
}

#[derive(Clone, Debug, Builder)]
#[builder(derive(Clone))]
pub struct BytesValidator {
  /// Specifies that the given `bytes` field must be of this exact length.
  pub len: Option<usize>,
  /// Specifies that the given `bytes` field must have a length that is equal to or higher than the given value.
  pub min_len: Option<usize>,
  /// Specifies that the given `bytes` field must have a length that is equal to or lower than the given value.
  pub max_len: Option<usize>,
  #[cfg(feature = "regex")]
  /// Specifies a regex pattern that must be matches by the value to pass validation.
  pub pattern: Option<&'static Regex>,
  /// Specifies a prefix that the value must start with in order to pass validation.
  pub prefix: Option<Bytes>,
  /// Specifies a suffix that the value must end with in order to pass validation.
  pub suffix: Option<Bytes>,
  /// Specifies a subset of bytes that the value must contain in order to pass validation.
  pub contains: Option<Bytes>,
  /// Specifies that only the values in this list will be considered valid for this field.
  #[builder(into)]
  pub in_: Option<ItemLookup<'static, &'static [u8]>>,
  /// Specifies that the values in this list will be considered NOT valid for this field.
  #[builder(into)]
  pub not_in: Option<ItemLookup<'static, &'static [u8]>>,
  #[builder(setters(vis = "", name = well_known))]
  pub well_known: Option<WellKnownBytes>,
  /// Specifies that only this specific value will be considered valid for this field.
  pub const_: Option<Bytes>,
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  #[builder(default, with = |programs: impl IntoIterator<Item = &'static LazyLock<CelProgram>>| collect_programs(programs))]
  pub cel: Vec<&'static CelProgram>,
  #[builder(default, with = || true)]
  /// Specifies that the field must be set in order to be valid.
  pub required: bool,
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

    #[cfg(feature = "regex")]
    if let Some(pattern) = validator.pattern {
      rules.push((
        PATTERN.clone(),
        OptionValue::String(pattern.as_str().into()),
      ))
    }

    insert_bytes_option!(validator, rules, contains);
    insert_bytes_option!(validator, rules, prefix);
    insert_bytes_option!(validator, rules, suffix);

    if let Some(allowed_list) = &validator.in_ {
      rules.push((
        IN_.clone(),
        OptionValue::new_list(
          allowed_list
            .iter()
            .map(|b| OptionValue::String(format_bytes_as_proto_string_literal(b).into())),
        ),
      ));
    }

    if let Some(forbidden_list) = &validator.not_in {
      rules.push((
        NOT_IN.clone(),
        OptionValue::new_list(
          forbidden_list
            .iter()
            .map(|b| OptionValue::String(format_bytes_as_proto_string_literal(b).into())),
        ),
      ));
    }

    if let Some(v) = validator.well_known {
      v.to_option(&mut rules);
    }

    let mut outer_rules: OptionValueList = vec![];

    outer_rules.push((BYTES.clone(), OptionValue::Message(rules.into())));

    insert_cel_rules!(validator, outer_rules);
    insert_boolean_option!(validator, outer_rules, required);
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
