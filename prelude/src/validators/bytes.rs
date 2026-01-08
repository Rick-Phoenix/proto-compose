use crate::validators::string::well_known_strings::*;
pub mod builder;
pub use builder::BytesValidatorBuilder;
use builder::state::State;

use ::bytes::Bytes;
#[cfg(feature = "regex")]
use regex::bytes::Regex;

use super::*;

impl_validator!(BytesValidator, Bytes);
impl_proto_type!(Bytes, Bytes);
impl_proto_type!(Vec<u8>, Bytes);

#[derive(Clone, Debug)]
pub struct BytesValidator {
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  pub cel: Vec<CelProgram>,

  pub ignore: Ignore,

  pub well_known: Option<WellKnownBytes>,

  /// Specifies that the field must be set in order to be valid.
  pub required: bool,

  /// Specifies that the given `bytes` field must be of this exact length.
  pub len: Option<usize>,

  /// Specifies that the given `bytes` field must have a length that is equal to or higher than the given value.
  pub min_len: Option<usize>,

  /// Specifies that the given `bytes` field must have a length that is equal to or lower than the given value.
  pub max_len: Option<usize>,

  #[cfg(feature = "regex")]
  /// Specifies a regex pattern that must be matches by the value to pass validation.
  pub pattern: Option<Regex>,

  /// Specifies a prefix that the value must start with in order to pass validation.
  pub prefix: Option<Bytes>,

  /// Specifies a suffix that the value must end with in order to pass validation.
  pub suffix: Option<Bytes>,

  /// Specifies a subset of bytes that the value must contain in order to pass validation.
  pub contains: Option<Bytes>,

  /// Specifies that only the values in this list will be considered valid for this field.
  pub in_: Option<StaticLookup<&'static [u8]>>,

  /// Specifies that the values in this list will be considered NOT valid for this field.
  pub not_in: Option<StaticLookup<&'static [u8]>>,

  /// Specifies that only this specific value will be considered valid for this field.
  pub const_: Option<Bytes>,
}

impl BytesValidator {
  #[inline]
  const fn has_pattern(&self) -> bool {
    #[cfg(feature = "regex")]
    {
      self.pattern.is_some()
    }
    #[cfg(not(feature = "regex"))]
    {
      false
    }
  }
}

pub(crate) struct LengthRuleValue {
  pub name: &'static str,
  pub value: Option<usize>,
}

pub(crate) fn check_length_rules(
  exact: Option<&LengthRuleValue>,
  min: &LengthRuleValue,
  max: &LengthRuleValue,
) -> Result<(), ConsistencyError> {
  if let Some(exact) = exact
    && exact.value.is_some()
  {
    if min.value.is_some() {
      return Err(ConsistencyError::ContradictoryInput(format!(
        "{} cannot be used with {}",
        exact.name, min.name
      )));
    }

    if max.value.is_some() {
      return Err(ConsistencyError::ContradictoryInput(format!(
        "{} cannot be used with {}",
        exact.name, max.name
      )));
    }
  }

  if let Some(min_value) = min.value
    && let Some(max_value) = max.value
    && min_value > max_value
  {
    return Err(ConsistencyError::ContradictoryInput(format!(
      "{} cannot be greater than {}",
      min.name, max.name
    )));
  }

  Ok(())
}

impl Validator<Bytes> for BytesValidator {
  type Target = Bytes;
  type UniqueStore<'a>
    = RefHybridStore<'a, Bytes>
  where
    Self: 'a;

  #[inline]
  fn make_unique_store<'a>(&self, cap: usize) -> Self::UniqueStore<'a> {
    RefHybridStore::default_with_capacity(cap)
  }

  fn check_consistency(&self) -> Result<(), Vec<ConsistencyError>> {
    let mut errors = Vec::new();

    macro_rules! check_prop_some {
      ($($id:ident),*) => {
        $(self.$id.is_some()) ||*
      };
    }

    if self.const_.is_some()
      && (!self.cel.is_empty()
        || check_prop_some!(
          len, min_len, max_len, prefix, suffix, contains, in_, not_in, well_known
        )
        || self.has_pattern())
    {
      errors.push(ConsistencyError::ConstWithOtherRules);
    }

    #[cfg(feature = "cel")]
    if let Err(e) = self.check_cel_programs() {
      errors.extend(e.into_iter().map(ConsistencyError::from));
    }

    if let Err(e) = check_list_rules(self.in_.as_ref(), self.not_in.as_ref()) {
      errors.push(e.into());
    }

    if let Err(e) = check_length_rules(
      Some(length_rule_value!("len", self.len)),
      length_rule_value!("min_len", self.min_len),
      length_rule_value!("max_len", self.max_len),
    ) {
      errors.push(e);
    }

    if errors.is_empty() {
      Ok(())
    } else {
      Err(errors)
    }
  }

  #[doc(hidden)]
  fn cel_rules(&self) -> Vec<CelRule> {
    self.cel.iter().map(|p| p.rule.clone()).collect()
  }

  #[cfg(feature = "cel")]
  fn check_cel_programs_with(&self, val: Self::Target) -> Result<(), Vec<CelError>> {
    if self.cel.is_empty() {
      Ok(())
    } else {
      // This one needs a special impl because Bytes does not support Into<Value>
      test_programs(&self.cel, val.to_vec())
    }
  }

  fn validate(&self, ctx: &mut ValidationCtx, val: Option<&Self::Target>) {
    handle_ignore_always!(&self.ignore);
    handle_ignore_if_zero_value!(&self.ignore, val.is_none_or(|v| v.is_default()));

    if self.required && val.is_none_or(|v| v.is_empty()) {
      ctx.add_required_violation();
    }

    if let Some(val) = val {
      if let Some(const_val) = &self.const_ {
        if *val != const_val {
          ctx.add_violation(
            &BYTES_CONST_VIOLATION,
            &format!("must be equal to {}", const_val.escape_ascii()),
          );
        }

        // Using `const` implies no other rules
        return;
      }

      if let Some(len) = self.len
        && val.len() != len
      {
        ctx.add_violation(
          &BYTES_LEN_VIOLATION,
          &format!("must be exactly {len} bytes long"),
        );
      }

      if let Some(min_len) = self.min_len
        && val.len() < min_len
      {
        ctx.add_violation(
          &BYTES_MIN_LEN_VIOLATION,
          &format!("must be at least {min_len} bytes long"),
        );
      }

      if let Some(max_len) = self.max_len
        && val.len() > max_len
      {
        ctx.add_violation(
          &BYTES_MAX_LEN_VIOLATION,
          &format!("cannot be longer than {max_len} bytes"),
        );
      }

      #[cfg(feature = "regex")]
      if let Some(pattern) = &self.pattern
        && !pattern.is_match(val)
      {
        ctx.add_violation(
          &BYTES_PATTERN_VIOLATION,
          &format!("must match the pattern `{pattern}`"),
        );
      }

      if let Some(prefix) = &self.prefix
        && !val.starts_with(prefix)
      {
        ctx.add_violation(
          &BYTES_PREFIX_VIOLATION,
          &format!("must start with {}", prefix.escape_ascii()),
        );
      }

      if let Some(suffix) = &self.suffix
        && !val.ends_with(suffix)
      {
        ctx.add_violation(
          &BYTES_SUFFIX_VIOLATION,
          &format!("must end with {}", suffix.escape_ascii()),
        );
      }

      if let Some(substring) = &self.contains
        && !val
          .windows(val.len())
          .any(|slice| slice == substring)
      {
        ctx.add_violation(
          &BYTES_CONTAINS_VIOLATION,
          &format!("must contain {}", substring.escape_ascii()),
        );
      }

      if let Some(allowed_list) = &self.in_
        && !allowed_list.items.contains(&val.as_ref())
      {
        let err = ["must be one of these values: ", &allowed_list.items_str].concat();

        ctx.add_violation(&BYTES_IN_VIOLATION, &err);
      }

      if let Some(forbidden_list) = &self.not_in
        && forbidden_list.items.contains(&val.as_ref())
      {
        let err = ["cannot be one of these values: ", &forbidden_list.items_str].concat();

        ctx.add_violation(&BYTES_IN_VIOLATION, &err);
      }

      if let Some(well_known) = &self.well_known {
        let byte_str = core::str::from_utf8(val.as_ref()).unwrap_or("");

        match well_known {
          #[cfg(feature = "regex")]
          WellKnownBytes::Uuid => {
            if !is_valid_uuid(byte_str) {
              ctx.add_violation(&BYTES_UUID_VIOLATION, "must be a valid UUID");
            }
          }
          WellKnownBytes::Ip => {
            if !is_valid_ip(byte_str) {
              ctx.add_violation(&BYTES_IP_VIOLATION, "must be a valid ip address");
            }
          }
          WellKnownBytes::Ipv4 => {
            if !is_valid_ipv4(byte_str) {
              ctx.add_violation(&BYTES_IPV4_VIOLATION, "must be a valid ipv4 address");
            }
          }
          WellKnownBytes::Ipv6 => {
            if !is_valid_ipv6(byte_str) {
              ctx.add_violation(&BYTES_IPV6_VIOLATION, "must be a valid ipv6 address");
            }
          }
        };
      }

      #[cfg(feature = "cel")]
      if !self.cel.is_empty() {
        let ctx = ProgramsExecutionCtx {
          programs: &self.cel,
          value: val.to_vec(),
          violations: ctx.violations,
          field_context: Some(&ctx.field_context),
          parent_elements: ctx.parent_elements,
        };

        ctx.execute_programs();
      }
    }
  }
}

impl From<BytesValidator> for ProtoOption {
  fn from(validator: BytesValidator) -> Self {
    let mut rules = OptionMessageBuilder::new();

    macro_rules! set_options {
      ($($name:ident),*) => {
        paste::paste! {
          rules
          $(
            .maybe_set(&[< $name:upper >], validator.$name)
          )*
        }
      };
    }

    set_options!(const_, min_len, max_len, len, contains, prefix, suffix);

    #[cfg(feature = "regex")]
    if let Some(pattern) = validator.pattern {
      rules.set(
        PATTERN.clone(),
        OptionValue::String(pattern.as_str().into()),
      );
    }

    rules
      .maybe_set(
        &IN_,
        validator.in_.map(|list| {
          OptionValue::new_list(
            list
              .items
              .iter()
              .map(|b| OptionValue::String(format_bytes_as_proto_string_literal(b).into())),
          )
        }),
      )
      .maybe_set(
        &NOT_IN,
        validator.not_in.map(|list| {
          OptionValue::new_list(
            list
              .items
              .iter()
              .map(|b| OptionValue::String(format_bytes_as_proto_string_literal(b).into())),
          )
        }),
      );

    if let Some(well_known) = validator.well_known {
      let (name, val) = well_known.to_option();
      rules.set(name, val);
    }

    let mut outer_rules = OptionMessageBuilder::new();

    outer_rules.set(BYTES.clone(), OptionValue::Message(rules.into()));

    outer_rules
      .add_cel_options(validator.cel)
      .set_required(validator.required)
      .set_ignore(validator.ignore);

    Self {
      name: BUF_VALIDATE_FIELD.clone(),
      value: OptionValue::Message(outer_rules.into()),
    }
  }
}

#[doc(hidden)]
#[derive(Clone, Debug, Copy)]
pub enum WellKnownBytes {
  Uuid,
  Ip,
  Ipv4,
  Ipv6,
}

impl WellKnownBytes {
  pub(crate) fn to_option(self) -> (Arc<str>, OptionValue) {
    let name = match self {
      Self::Uuid => UUID.clone(),
      Self::Ip => IP.clone(),
      Self::Ipv4 => IPV4.clone(),
      Self::Ipv6 => IPV6.clone(),
    };

    (name, OptionValue::Bool(true))
  }
}

pub(crate) fn format_bytes_as_proto_string_literal(bytes: &[u8]) -> String {
  let mut result = String::new();

  for &byte in bytes {
    match byte {
      0x20..=0x21 | 0x23..=0x5B | 0x5D..=0x7E => {
        result.push(byte as char);
      }
      b'\\' => result.push_str("\\\\"),
      b'"' => result.push_str("\\\""),
      _ => {
        let _ = write!(result, "\\x{byte:02x}");
      }
    }
  }

  result
}
