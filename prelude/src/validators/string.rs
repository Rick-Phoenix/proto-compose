pub(crate) mod well_known_strings;
use well_known_strings::*;
mod builder;
pub use builder::StringValidatorBuilder;

#[cfg(feature = "regex")]
use regex::Regex;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum FixedStr {
  Static(&'static str),
  Shared(Arc<str>),
  Boxed(Box<str>),
}

impl Default for FixedStr {
  #[inline]
  fn default() -> Self {
    Self::Static("")
  }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for FixedStr {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;

    Ok(Self::from(s))
  }
}

impl FixedStr {
  #[inline]
  #[must_use]
  pub fn into_cheaply_clonable(self) -> Self {
    match self {
      Self::Static(_) | Self::Shared(_) => self,
      Self::Boxed(boxed) => Self::Shared(boxed.into()),
    }
  }

  #[must_use]
  pub fn as_str(&self) -> &str {
    match self {
      Self::Static(s) => s,
      Self::Shared(s) => s,
      Self::Boxed(s) => s,
    }
  }
}

impl From<FixedStr> for String {
  fn from(value: FixedStr) -> Self {
    value.to_string()
  }
}

impl Borrow<str> for FixedStr {
  fn borrow(&self) -> &str {
    self
  }
}

impl AsRef<str> for FixedStr {
  fn as_ref(&self) -> &str {
    self
  }
}

impl<'a> PartialEq<&'a str> for FixedStr {
  fn eq(&self, other: &&'a str) -> bool {
    **other == **self
  }
}

impl Display for FixedStr {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "{}", self.as_str())
  }
}

impl core::ops::Deref for FixedStr {
  type Target = str;

  fn deref(&self) -> &Self::Target {
    match self {
      Self::Static(str) => str,
      Self::Shared(str) => str,
      Self::Boxed(str) => str,
    }
  }
}

impl From<&'static str> for FixedStr {
  fn from(value: &'static str) -> Self {
    Self::Static(value)
  }
}

impl From<Box<str>> for FixedStr {
  fn from(value: Box<str>) -> Self {
    Self::Boxed(value)
  }
}

impl From<String> for FixedStr {
  fn from(value: String) -> Self {
    Self::Boxed(value.into_boxed_str())
  }
}

impl From<Arc<str>> for FixedStr {
  fn from(value: Arc<str>) -> Self {
    Self::Shared(value)
  }
}

impl From<&Arc<str>> for FixedStr {
  fn from(value: &Arc<str>) -> Self {
    Self::Shared(value.clone())
  }
}

#[non_exhaustive]
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StringValidator {
  /// Adds custom validation using one or more [`CelRule`]s to this field.
  pub cel: Vec<CelProgram>,

  pub well_known: Option<WellKnownStrings>,

  pub ignore: Ignore,

  /// Specifies that the field must be set in order to be valid.
  pub required: bool,

  /// Specifies that the given string field must be of this exact length.
  pub len: Option<usize>,

  /// Specifies that the given string field must have a length that is equal to or higher than the given value.
  pub min_len: Option<usize>,

  /// Specifies that the given string field must have a length that is equal to or lower than the given value.
  pub max_len: Option<usize>,

  /// Specifies the exact byte length that this field's value must have in order to be considered valid.
  pub len_bytes: Option<usize>,

  /// Specifies the minimum byte length for this field's value to be considered valid.
  pub min_bytes: Option<usize>,

  /// Specifies the minimum byte length for this field's value to be considered valid.
  pub max_bytes: Option<usize>,

  #[cfg(feature = "regex")]
  #[cfg_attr(feature = "serde", serde(with = "crate::serde_impls::regex_serde"))]
  /// Specifies a regex pattern that this field's value should match in order to be considered valid.
  pub pattern: Option<Regex>,

  /// Specifies the prefix that this field's value should contain in order to be considered valid.
  pub prefix: Option<FixedStr>,

  /// Specifies the suffix that this field's value should contain in order to be considered valid.
  pub suffix: Option<FixedStr>,

  /// Specifies a substring that this field's value should contain in order to be considered valid.
  pub contains: Option<FixedStr>,

  /// Specifies a substring that this field's value must not contain in order to be considered valid.
  pub not_contains: Option<FixedStr>,

  /// Specifies that only the values in this list will be considered valid for this field.
  pub in_: Option<SortedList<FixedStr>>,

  /// Specifies that the values in this list will be considered NOT valid for this field.
  pub not_in: Option<SortedList<FixedStr>>,

  /// Specifies that only this specific value will be considered valid for this field.
  pub const_: Option<FixedStr>,

  pub error_messages: Option<ErrorMessages<StringViolation>>,
}

impl StringValidator {
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

impl_proto_type!(String, String);
impl_proto_map_key!(String, String);

impl Validator<String> for StringValidator {
  type Target = str;

  #[cfg(feature = "cel")]
  fn check_cel_programs_with(
    &self,
    val: <Self::Target as ToOwned>::Owned,
  ) -> Result<(), Vec<CelError>> {
    if self.cel.is_empty() {
      Ok(())
    } else {
      test_programs(&self.cel, val)
    }
  }
  #[cfg(feature = "cel")]
  fn check_cel_programs(&self) -> Result<(), Vec<CelError>> {
    self.check_cel_programs_with(String::new())
  }
  #[doc(hidden)]
  fn cel_rules(&self) -> Vec<CelRule> {
    self.cel.iter().map(|p| p.rule.clone()).collect()
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
          in_,
          not_in,
          well_known,
          len,
          min_len,
          max_len,
          len_bytes,
          min_bytes,
          max_bytes,
          suffix,
          prefix,
          contains,
          not_contains
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
                StringViolation::Required => self.required,
                StringViolation::In => self.in_.is_some(),
                StringViolation::Const => self.const_.is_some(),
                StringViolation::WellKnownRegex => self.well_known.is_some(),
                #[cfg(feature = "regex")]
                StringViolation::Pattern => self.pattern.is_some(),
                $(StringViolation::[< $name:camel >] => self.$name.is_some(),)*
                _ => true,
              }
            }
          };
        }

        let is_used = check_unused_messages!(
          len,
          min_len,
          max_len,
          len_bytes,
          min_bytes,
          max_bytes,
          prefix,
          suffix,
          contains,
          not_contains,
          not_in
        );

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

    if let Some(forbidden_substr) = self.not_contains.as_deref() {
      if let Some(required_substr) = self.contains.as_deref()
        && required_substr.contains(forbidden_substr)
      {
        errors.push(ConsistencyError::ContradictoryInput(
          "`not_contains` is a substring of `contains`".to_string(),
        ));
      }

      if let Some(prefix) = self.prefix.as_deref()
        && prefix.contains(forbidden_substr)
      {
        errors.push(ConsistencyError::ContradictoryInput(
          "`not_contains` is a substring of `prefix`".to_string(),
        ));
      }

      if let Some(suffix) = self.suffix.as_deref()
        && suffix.contains(forbidden_substr)
      {
        errors.push(ConsistencyError::ContradictoryInput(
          "`not_contains` is a substring of `suffix`".to_string(),
        ));
      }

      if let Some(allowed_values) = self.in_.as_ref() {
        for str in allowed_values {
          if str.contains(forbidden_substr) {
            errors.push(ConsistencyError::ContradictoryInput(
              format!("The `in` list contains '{str}', which matches the `not_contains` substring '{forbidden_substr}'")
            ));
          }
        }
      }
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

    if let Err(e) = check_length_rules(
      Some(length_rule_value!("len_bytes", self.len_bytes)),
      length_rule_value!("min_bytes", self.min_bytes),
      length_rule_value!("max_bytes", self.max_bytes),
    ) {
      errors.push(e);
    }

    if errors.is_empty() {
      Ok(())
    } else {
      Err(errors)
    }
  }

  fn validate_core<V>(&self, ctx: &mut ValidationCtx, val: Option<&V>) -> ValidatorResult
  where
    V: Borrow<Self::Target> + ?Sized,
  {
    handle_ignore_always!(&self.ignore);
    handle_ignore_if_zero_value!(&self.ignore, val.is_none_or(|v| v.borrow().is_empty()));

    let mut is_valid = IsValid::Yes;

    macro_rules! handle_violation {
      ($id:ident, $default:expr) => {
        is_valid &= ctx.add_violation(
          ViolationKind::String(StringViolation::$id),
          self
            .error_messages
            .as_deref()
            .and_then(|map| map.get(&StringViolation::$id))
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
        if val != const_val.as_ref() {
          handle_violation!(Const, format!("must be equal to {const_val}"));
        }

        // Using `const` implies no other rules
        return Ok(is_valid);
      }

      if let Some(len) = self.len
        && val.chars().count() != len
      {
        handle_violation!(
          Len,
          format!("must be exactly {len} character{} long", pluralize!(len))
        );
      }

      if let Some(min_len) = self.min_len
        && val.chars().count() < min_len
      {
        handle_violation!(
          MinLen,
          format!(
            "must be at least {min_len} character{} long",
            pluralize!(min_len)
          )
        );
      }

      if let Some(max_len) = self.max_len
        && val.chars().count() > max_len
      {
        handle_violation!(
          MaxLen,
          format!(
            "cannot be longer than {max_len} character{}",
            pluralize!(max_len)
          )
        );
      }

      if let Some(len_bytes) = self.len_bytes
        && val.len() != len_bytes
      {
        handle_violation!(
          LenBytes,
          format!(
            "must be exactly {len_bytes} byte{} long",
            pluralize!(len_bytes)
          )
        );
      }

      if let Some(min_bytes) = self.min_bytes
        && val.len() < min_bytes
      {
        handle_violation!(
          MinBytes,
          format!(
            "must be at least {min_bytes} byte{} long",
            pluralize!(min_bytes)
          )
        );
      }

      if let Some(max_bytes) = self.max_bytes
        && val.len() > max_bytes
      {
        handle_violation!(
          MaxBytes,
          format!(
            "cannot be longer than {max_bytes} byte{}",
            pluralize!(max_bytes)
          )
        );
      }

      if let Some(prefix) = &self.prefix
        && !val.starts_with(&**prefix)
      {
        handle_violation!(Prefix, format!("must start with {prefix}"));
      }

      if let Some(suffix) = &self.suffix
        && !val.ends_with(&**suffix)
      {
        handle_violation!(Suffix, format!("must end with {suffix}"));
      }

      if let Some(substring) = &self.contains
        && !val.contains(substring.as_ref())
      {
        handle_violation!(Contains, format!("must contain {substring}"));
      }

      if let Some(substring) = &self.not_contains
        && val.contains(substring.as_ref())
      {
        handle_violation!(NotContains, format!("cannot contain {substring}"));
      }

      #[cfg(feature = "regex")]
      if let Some(pattern) = &self.pattern
        && !pattern.is_match(val)
      {
        handle_violation!(Pattern, format!("must match the pattern `{pattern}`"));
      }

      if let Some(allowed_list) = &self.in_
        && !allowed_list.contains(val)
      {
        handle_violation!(
          In,
          format!(
            "must be one of these values: {}",
            FixedStr::format_list(allowed_list)
          )
        );
      }

      if let Some(forbidden_list) = &self.not_in
        && forbidden_list.contains(val)
      {
        handle_violation!(
          NotIn,
          format!(
            "cannot be one of these values: {}",
            FixedStr::format_list(forbidden_list)
          )
        );
      }

      macro_rules! impl_well_known_check {
        ($check:expr, $violation:ident, $msg:literal) => {
          if !$check(val.as_ref()) {
            handle_violation!($violation, format!("must be a valid {}", $msg));
          }
        };
      }

      if let Some(well_known) = &self.well_known {
        match well_known {
          #[cfg(feature = "regex")]
          WellKnownStrings::Ulid => {
            impl_well_known_check!(is_valid_ulid, Ulid, "ULID");
          }
          WellKnownStrings::Ip => {
            impl_well_known_check!(is_valid_ip, Ip, "ip address");
          }
          WellKnownStrings::Ipv4 => {
            impl_well_known_check!(is_valid_ipv4, Ipv4, "ipv4 address");
          }
          WellKnownStrings::Ipv6 => {
            impl_well_known_check!(is_valid_ipv6, Ipv6, "ipv6 address");
          }
          #[cfg(feature = "regex")]
          WellKnownStrings::Email => {
            impl_well_known_check!(is_valid_email, Email, "email address");
          }
          WellKnownStrings::Hostname => {
            impl_well_known_check!(is_valid_hostname, Hostname, "hostname");
          }
          WellKnownStrings::Uri => {
            impl_well_known_check!(is_valid_uri, Uri, "uri");
          }
          WellKnownStrings::UriRef => {
            impl_well_known_check!(is_valid_uri_ref, UriRef, "uri reference");
          }
          WellKnownStrings::Address => {
            impl_well_known_check!(is_valid_address, Address, "address");
          }
          #[cfg(feature = "regex")]
          WellKnownStrings::Uuid => {
            impl_well_known_check!(is_valid_uuid, Uuid, "uuid");
          }
          #[cfg(feature = "regex")]
          WellKnownStrings::Tuuid => {
            impl_well_known_check!(is_valid_tuuid, Tuuid, "trimmed uuid");
          }
          WellKnownStrings::IpWithPrefixlen => {
            impl_well_known_check!(
              is_valid_ip_with_prefixlen,
              IpWithPrefixlen,
              "ip with prefix length"
            );
          }
          WellKnownStrings::Ipv4WithPrefixlen => {
            impl_well_known_check!(
              is_valid_ipv4_with_prefixlen,
              Ipv4WithPrefixlen,
              "ipv4 with prefix length"
            );
          }
          WellKnownStrings::Ipv6WithPrefixlen => {
            impl_well_known_check!(
              is_valid_ipv6_with_prefixlen,
              Ipv6WithPrefixlen,
              "ipv6 with prefix length"
            );
          }
          WellKnownStrings::IpPrefix => {
            impl_well_known_check!(is_valid_ip_prefix, IpPrefix, "ip prefix");
          }
          WellKnownStrings::Ipv4Prefix => {
            impl_well_known_check!(is_valid_ipv4_prefix, Ipv4Prefix, "ipv4 prefix");
          }
          WellKnownStrings::Ipv6Prefix => {
            impl_well_known_check!(is_valid_ipv6_prefix, Ipv6Prefix, "ipv6 prefix");
          }
          WellKnownStrings::HostAndPort => {
            impl_well_known_check!(is_valid_host_and_port, HostAndPort, "host and port");
          }
          #[cfg(feature = "regex")]
          WellKnownStrings::HeaderNameLoose => {
            if !is_valid_http_header_name(val, false) {
              handle_violation!(
                WellKnownRegex,
                "must be a valid http header name".to_string()
              );
            }
          }
          #[cfg(feature = "regex")]
          WellKnownStrings::HeaderNameStrict => {
            if !is_valid_http_header_name(val, true) {
              handle_violation!(
                WellKnownRegex,
                "must be a valid http header name".to_string()
              );
            }
          }
          #[cfg(feature = "regex")]
          WellKnownStrings::HeaderValueLoose => {
            if !is_valid_http_header_value(val, false) {
              handle_violation!(
                WellKnownRegex,
                "must be a valid http header value".to_string()
              );
            }
          }
          #[cfg(feature = "regex")]
          WellKnownStrings::HeaderValueStrict => {
            if !is_valid_http_header_value(val, true) {
              handle_violation!(
                WellKnownRegex,
                "must be a valid http header value".to_string()
              );
            }
          }
        };
      }

      #[cfg(feature = "cel")]
      if !self.cel.is_empty() {
        let cel_ctx = ProgramsExecutionCtx {
          programs: &self.cel,
          value: val,
          ctx,
        };

        is_valid &= cel_ctx.execute_programs()?;
      }
    }

    Ok(is_valid)
  }

  fn schema(&self) -> Option<ValidatorSchema> {
    Some(ValidatorSchema {
      schema: self.clone().into(),
      cel_rules: self.cel_rules(),
      imports: vec!["buf/validate/validate.proto".into()],
    })
  }
}

impl From<StringValidator> for ProtoOption {
  fn from(validator: StringValidator) -> Self {
    let mut rules = OptionMessageBuilder::new();

    macro_rules! set_options {
      ($($name:ident),*) => {
        rules
        $(
          .maybe_set(stringify!($name), validator.$name)
        )*
      };
    }

    set_options!(
      min_len,
      max_len,
      len,
      min_bytes,
      max_bytes,
      len_bytes,
      prefix,
      suffix,
      contains,
      not_contains
    );

    #[cfg(feature = "regex")]
    if let Some(pattern) = validator.pattern {
      rules.set("pattern", OptionValue::String(pattern.to_string().into()));
    }

    rules
      .maybe_set("const", validator.const_)
      .maybe_set(
        "in",
        validator
          .in_
          .map(|list| OptionValue::new_list(list)),
      )
      .maybe_set(
        "not_in",
        validator
          .not_in
          .map(|list| OptionValue::new_list(list)),
      );

    if let Some(well_known) = validator.well_known {
      let (name, value, is_strict) = well_known.to_option();

      rules.set(name, value);
      rules.set_boolean("strict", is_strict);
    }

    // This is the outer rule grouping, "(buf.validate.field)"
    let mut outer_rules = OptionMessageBuilder::new();

    // (buf.validate.field).string .et_cetera...
    if !rules.is_empty() {
      outer_rules.set("string", OptionValue::Message(rules.into()));
    }

    // These must be added on the outer grouping, as they are generic rules
    // It's (buf.validate.field).required, NOT (buf.validate.field).string.required
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
