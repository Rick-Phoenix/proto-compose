use crate::validators::string::well_known_strings::*;
mod builder;
pub use builder::BytesValidatorBuilder;

use ::bytes::Bytes;
#[cfg(feature = "regex")]
use regex::bytes::Regex;

use super::*;

impl_proto_type!(Bytes, Bytes);
impl_proto_type!(Vec<u8>, Bytes);

#[non_exhaustive]
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
  #[cfg_attr(
    feature = "serde",
    serde(with = "crate::serde_impls::bytes_regex_serde")
  )]
  /// Specifies a regex pattern that must be matches by the value to pass validation.
  pub pattern: Option<Regex>,

  /// Specifies a prefix that the value must start with in order to pass validation.
  pub prefix: Option<Bytes>,

  /// Specifies a suffix that the value must end with in order to pass validation.
  pub suffix: Option<Bytes>,

  /// Specifies a subset of bytes that the value must contain in order to pass validation.
  pub contains: Option<Bytes>,

  /// Specifies that only the values in this list will be considered valid for this field.
  pub in_: Option<SortedList<Bytes>>,

  /// Specifies that the values in this list will be considered NOT valid for this field.
  pub not_in: Option<SortedList<Bytes>>,

  /// Specifies that only this specific value will be considered valid for this field.
  pub const_: Option<Bytes>,

  pub error_messages: Option<ErrorMessages<BytesViolation>>,
}

impl BytesValidator {
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

impl Validator<Bytes> for BytesValidator {
  type Target = Bytes;

  #[inline(never)]
  #[cold]
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

    if let Some(custom_messages) = self.error_messages.as_deref() {
      let mut unused_messages: Vec<String> = Vec::new();

      for key in custom_messages.keys() {
        macro_rules! check_unused_messages {
          ($($name:ident),*) => {
            paste! {
              match key {
                BytesViolation::Required => self.required,
                BytesViolation::In => self.in_.is_some(),
                BytesViolation::Const => self.const_.is_some(),
                BytesViolation::Ip
                | BytesViolation::Ipv4
                | BytesViolation::Ipv6
                | BytesViolation::Uuid => self.well_known.is_some(),
                #[cfg(feature = "regex")]
                BytesViolation::Pattern => self.pattern.is_some(),
                $(BytesViolation::[< $name:camel >] => self.$name.is_some(),)*
                _ => true,
              }
            }
          };
        }

        let is_used =
          check_unused_messages!(len, min_len, max_len, contains, prefix, suffix, not_in);

        if !is_used {
          unused_messages.push(format!("{key:?}"));
        }
      }

      if !unused_messages.is_empty() {
        errors.push(ConsistencyError::UnusedCustomMessages(unused_messages));
      }
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
  #[inline(never)]
  #[cold]
  fn check_cel_programs(&self) -> Result<(), Vec<CelError>> {
    self.check_cel_programs_with(Bytes::default())
  }

  #[cfg(feature = "cel")]
  #[inline(never)]
  #[cold]
  fn check_cel_programs_with(&self, val: Self::Target) -> Result<(), Vec<CelError>> {
    if self.cel.is_empty() {
      Ok(())
    } else {
      // This one needs a special impl because Bytes does not support Into<Value>
      test_programs(&self.cel, val.to_vec())
    }
  }

  fn validate_core<V>(&self, ctx: &mut ValidationCtx, val: Option<&V>) -> ValidationResult
  where
    V: Borrow<Self::Target> + ?Sized,
  {
    handle_ignore_always!(&self.ignore);
    handle_ignore_if_zero_value!(&self.ignore, val.is_none_or(|v| v.borrow().is_empty()));

    let mut is_valid = IsValid::Yes;

    macro_rules! handle_violation {
      ($id:ident, $default:expr) => {
        is_valid &= ctx.add_violation(
          ViolationKind::Bytes(BytesViolation::$id),
          self
            .error_messages
            .as_deref()
            .and_then(|map| map.get(&BytesViolation::$id))
            .map(|m| Cow::Borrowed(m.as_ref()))
            .unwrap_or_else(|| Cow::Owned($default)),
        )?;
      };
    }

    if self.required && val.is_none_or(|v| v.borrow().is_empty()) {
      handle_violation!(Required, "is required".to_string());
      return Ok(is_valid);
    }

    if let Some(val) = val {
      let val = val.borrow();

      if let Some(const_val) = &self.const_ {
        if *val != const_val {
          handle_violation!(
            Const,
            format!("must be equal to {}", const_val.escape_ascii())
          );
        }

        // Using `const` implies no other rules
        return Ok(is_valid);
      }

      if let Some(len) = self.len
        && val.len() != len
      {
        handle_violation!(
          Len,
          format!("must be exactly {len} byte{} long", pluralize!(len))
        );
      }

      if let Some(min_len) = self.min_len
        && val.len() < min_len
      {
        handle_violation!(
          MinLen,
          format!(
            "must be at least {min_len} byte{} long",
            pluralize!(min_len)
          )
        );
      }

      if let Some(max_len) = self.max_len
        && val.len() > max_len
      {
        handle_violation!(
          MaxLen,
          format!(
            "cannot be longer than {max_len} byte{}",
            pluralize!(max_len)
          )
        );
      }

      if let Some(prefix) = &self.prefix
        && !val.starts_with(prefix)
      {
        handle_violation!(Prefix, format!("must start with {}", prefix.escape_ascii()));
      }

      if let Some(suffix) = &self.suffix
        && !val.ends_with(suffix)
      {
        handle_violation!(Suffix, format!("must end with {}", suffix.escape_ascii()));
      }

      if let Some(substring) = &self.contains
        && !val
          .windows(val.len())
          .any(|slice| slice == substring)
      {
        handle_violation!(
          Contains,
          format!("must contain {}", substring.escape_ascii())
        );
      }

      #[cfg(feature = "regex")]
      if let Some(pattern) = &self.pattern
        && !pattern.is_match(val)
      {
        handle_violation!(Pattern, format!("must match the pattern `{pattern}`"));
      }

      if let Some(allowed_list) = &self.in_
        && !allowed_list.contains(val.as_ref())
      {
        handle_violation!(
          In,
          format!(
            "must be one of these values: {}",
            Bytes::format_list(allowed_list)
          )
        );
      }

      if let Some(forbidden_list) = &self.not_in
        && forbidden_list.contains(val.as_ref())
      {
        handle_violation!(
          NotIn,
          format!(
            "cannot be one of these values: {}",
            Bytes::format_list(forbidden_list)
          )
        );
      }

      if let Some(well_known) = &self.well_known {
        let byte_str = core::str::from_utf8(val.as_ref()).unwrap_or("");

        match well_known {
          #[cfg(feature = "regex")]
          WellKnownBytes::Uuid => {
            if !is_valid_uuid(byte_str) {
              handle_violation!(Uuid, "must be a valid UUID".to_string());
            }
          }
          WellKnownBytes::Ip => {
            if !is_valid_ip(byte_str) {
              handle_violation!(Ip, "must be a valid ip address".to_string());
            }
          }
          WellKnownBytes::Ipv4 => {
            if !is_valid_ipv4(byte_str) {
              handle_violation!(Ipv4, "must be a valid ipv4 address".to_string());
            }
          }
          WellKnownBytes::Ipv6 => {
            if !is_valid_ipv6(byte_str) {
              handle_violation!(Ipv6, "must be a valid ipv6 address".to_string());
            }
          }
        };
      }

      #[cfg(feature = "cel")]
      if !self.cel.is_empty() {
        let cel_ctx = ProgramsExecutionCtx {
          programs: &self.cel,
          value: val.to_vec(),
          ctx,
        };

        is_valid &= cel_ctx.execute_programs()?;
      }
    }

    Ok(is_valid)
  }

  #[inline(never)]
  #[cold]
  fn schema(&self) -> Option<ValidatorSchema> {
    Some(ValidatorSchema {
      schema: self.clone().into(),
      cel_rules: self.cel_rules(),
      imports: vec!["buf/validate/validate.proto".into()],
    })
  }
}

impl From<BytesValidator> for ProtoOption {
  #[inline(never)]
  #[cold]
  fn from(validator: BytesValidator) -> Self {
    let mut rules = OptionMessageBuilder::new();

    macro_rules! set_options {
      ($($name:ident),*) => {
        rules
        $(
          .maybe_set(stringify!($name), validator.$name)
        )*
      };
    }

    set_options!(min_len, max_len, len, contains, prefix, suffix);

    #[cfg(feature = "regex")]
    if let Some(pattern) = validator.pattern {
      rules.set("pattern", OptionValue::String(pattern.to_string().into()));
    }

    rules
      .maybe_set("const", validator.const_)
      .maybe_set(
        "in",
        validator.in_.map(|list| {
          OptionValue::List(
            list
              .items
              .iter()
              .map(|b| OptionValue::String(format_bytes_as_proto_string_literal(b).into()))
              .collect(),
          )
        }),
      )
      .maybe_set(
        "not_in",
        validator.not_in.map(|list| {
          OptionValue::List(
            list
              .items
              .iter()
              .map(|b| OptionValue::String(format_bytes_as_proto_string_literal(b).into()))
              .collect(),
          )
        }),
      );

    if let Some(well_known) = validator.well_known {
      let (name, val) = well_known.to_option();
      rules.set(name, val);
    }

    let mut outer_rules = OptionMessageBuilder::new();

    if !rules.is_empty() {
      outer_rules.set("bytes", OptionValue::Message(rules.into()));
    }

    outer_rules
      .add_cel_options(validator.cel)
      .set_required(validator.required)
      .set_ignore(validator.ignore);

    Self {
      name: "(buf.validate.field)".into(),
      value: OptionValue::Message(outer_rules.into()),
    }
  }
}

#[doc(hidden)]
#[non_exhaustive]
#[derive(Clone, Debug, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WellKnownBytes {
  #[cfg(feature = "regex")]
  Uuid,
  Ip,
  Ipv4,
  Ipv6,
}

impl WellKnownBytes {
  #[inline(never)]
  #[cold]
  pub(crate) fn to_option(self) -> (FixedStr, OptionValue) {
    let name = match self {
      #[cfg(feature = "regex")]
      Self::Uuid => "uuid",
      Self::Ip => "ip",
      Self::Ipv4 => "ipv4",
      Self::Ipv6 => "ipv6",
    };

    (name.into(), OptionValue::Bool(true))
  }
}

#[inline(never)]
#[cold]
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
