pub(crate) mod well_known_strings;
use well_known_strings::*;
mod builder;
pub use builder::StringValidatorBuilder;

#[cfg(feature = "regex")]
use regex::Regex;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SharedStr {
  Static(&'static str),
  Shared(Arc<str>),
}

impl SharedStr {
  #[must_use]
  pub fn as_str(&self) -> &str {
    match self {
      Self::Static(s) => s,
      Self::Shared(s) => s,
    }
  }
}

impl Borrow<str> for SharedStr {
  fn borrow(&self) -> &str {
    self
  }
}

impl AsRef<str> for SharedStr {
  fn as_ref(&self) -> &str {
    self
  }
}

impl<'a> PartialEq<&'a str> for SharedStr {
  fn eq(&self, other: &&'a str) -> bool {
    **other == **self
  }
}

impl Display for SharedStr {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "{}", self.as_str())
  }
}

impl core::ops::Deref for SharedStr {
  type Target = str;

  fn deref(&self) -> &Self::Target {
    match self {
      Self::Static(str) => str,
      Self::Shared(arc) => arc,
    }
  }
}

impl From<&'static str> for SharedStr {
  fn from(value: &'static str) -> Self {
    Self::Static(value)
  }
}

impl From<String> for SharedStr {
  fn from(value: String) -> Self {
    Self::Shared(value.into())
  }
}

impl From<Arc<str>> for SharedStr {
  fn from(value: Arc<str>) -> Self {
    Self::Shared(value)
  }
}

#[derive(Clone, Debug)]
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
  /// Specifies a regex pattern that this field's value should match in order to be considered valid.
  pub pattern: Option<Cow<'static, Regex>>,

  /// Specifies the prefix that this field's value should contain in order to be considered valid.
  pub prefix: Option<SharedStr>,

  /// Specifies the suffix that this field's value should contain in order to be considered valid.
  pub suffix: Option<SharedStr>,

  /// Specifies a substring that this field's value should contain in order to be considered valid.
  pub contains: Option<SharedStr>,

  /// Specifies a substring that this field's value must not contain in order to be considered valid.
  pub not_contains: Option<SharedStr>,

  /// Specifies that only the values in this list will be considered valid for this field.
  pub in_: Option<StaticLookup<SharedStr>>,

  /// Specifies that the values in this list will be considered NOT valid for this field.
  pub not_in: Option<StaticLookup<SharedStr>>,

  /// Specifies that only this specific value will be considered valid for this field.
  pub const_: Option<SharedStr>,
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
  type Target = String;
  type UniqueStore<'a>
    = RefHybridStore<'a, String>
  where
    Self: 'a;

  #[inline]
  fn make_unique_store<'a>(&self, cap: usize) -> Self::UniqueStore<'a> {
    RefHybridStore::default_with_capacity(cap)
  }

  impl_testing_methods!();

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

    #[cfg(feature = "cel")]
    if let Err(e) = self.check_cel_programs() {
      errors.extend(e.into_iter().map(ConsistencyError::from));
    }

    if let Some(contains) = self.contains.as_ref()
      && let Some(not_contains) = self.not_contains.as_ref()
      && contains == not_contains
    {
      errors.push(ConsistencyError::ContradictoryInput(
        "`contains` and `not_contains` have the same value".to_string(),
      ));
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

  fn validate(&self, ctx: &mut ValidationCtx, val: Option<&Self::Target>) {
    handle_ignore_always!(&self.ignore);
    handle_ignore_if_zero_value!(&self.ignore, val.is_none_or(|v| v.is_empty()));

    if self.required && val.is_none_or(|v| v.is_empty()) {
      ctx.add_required_violation();
    }

    if let Some(val) = val {
      if let Some(const_val) = &self.const_ {
        if val != const_val.as_ref() {
          ctx.add_violation(
            &STRING_CONST_VIOLATION,
            &format!("must be equal to {const_val}",),
          );
        }

        // Using `const` implies no other rules
        return;
      }

      if let Some(len) = self.len
        && val.chars().count() != len
      {
        ctx.add_violation(
          &STRING_LEN_VIOLATION,
          &format!("must be exactly {len} characters long"),
        );
      }

      if let Some(min_len) = self.min_len
        && val.chars().count() < min_len
      {
        ctx.add_violation(
          &STRING_MIN_LEN_VIOLATION,
          &format!("must be at least {min_len} characters long"),
        );
      }

      if let Some(max_len) = self.max_len
        && val.chars().count() > max_len
      {
        ctx.add_violation(
          &STRING_MAX_LEN_VIOLATION,
          &format!("cannot be longer than {max_len} characters"),
        );
      }

      if let Some(len_bytes) = self.len_bytes
        && val.len() != len_bytes
      {
        ctx.add_violation(
          &STRING_LEN_BYTES_VIOLATION,
          &format!("must be exactly {len_bytes} bytes long"),
        );
      }

      if let Some(min_bytes) = self.min_bytes
        && val.len() < min_bytes
      {
        ctx.add_violation(
          &STRING_MIN_BYTES_VIOLATION,
          &format!("must be at least {min_bytes} bytes long"),
        );
      }

      if let Some(max_bytes) = self.max_bytes
        && val.len() > max_bytes
      {
        ctx.add_violation(
          &STRING_MAX_BYTES_VIOLATION,
          &format!("cannot be longer than {max_bytes} bytes"),
        );
      }

      #[cfg(feature = "regex")]
      if let Some(pattern) = &self.pattern
        && !pattern.is_match(val)
      {
        ctx.add_violation(
          &STRING_PATTERN_VIOLATION,
          &format!("must match the pattern `{pattern}`"),
        );
      }

      if let Some(prefix) = &self.prefix
        && !val.starts_with(&**prefix)
      {
        ctx.add_violation(
          &STRING_PREFIX_VIOLATION,
          &format!("must start with {prefix}"),
        );
      }

      if let Some(suffix) = &self.suffix
        && !val.ends_with(&**suffix)
      {
        ctx.add_violation(&STRING_SUFFIX_VIOLATION, &format!("must end with {suffix}"));
      }

      if let Some(substring) = &self.contains
        && !val.contains(substring.as_ref())
      {
        ctx.add_violation(
          &STRING_CONTAINS_VIOLATION,
          &format!("must contain {substring}"),
        );
      }

      if let Some(substring) = &self.not_contains
        && val.contains(substring.as_ref())
      {
        ctx.add_violation(
          &STRING_NOT_CONTAINS_VIOLATION,
          &format!("cannot contain {substring}"),
        );
      }

      if let Some(allowed_list) = &self.in_
        && !allowed_list.items.contains(val.as_str())
      {
        let err = ["must be one of these values: ", &allowed_list.items_str].concat();

        ctx.add_violation(&STRING_IN_VIOLATION, &err);
      }

      if let Some(forbidden_list) = &self.not_in
        && forbidden_list.items.contains(val.as_str())
      {
        let err = ["cannot be one of these values: ", &forbidden_list.items_str].concat();

        ctx.add_violation(&STRING_NOT_IN_VIOLATION, &err);
      }

      macro_rules! impl_well_known_check {
        ($check:expr, $violation:ident, $msg:literal) => {
          paste::paste! {
            if !$check(val.as_ref()) {
              ctx.add_violation(
                &[< STRING_ $violation _VIOLATION >],
                concat!("must be a valid ", $msg),
              );
            }
          }
        };
      }

      if let Some(well_known) = &self.well_known {
        match well_known {
          #[cfg(feature = "regex")]
          WellKnownStrings::Ulid => {
            impl_well_known_check!(is_valid_ulid, ULID, "ULID");
          }
          WellKnownStrings::Ip => {
            impl_well_known_check!(is_valid_ip, IP, "ip address");
          }
          WellKnownStrings::Ipv4 => {
            impl_well_known_check!(is_valid_ipv4, IPV4, "ipv4 address");
          }
          WellKnownStrings::Ipv6 => {
            impl_well_known_check!(is_valid_ipv6, IPV6, "ipv6 address");
          }
          #[cfg(feature = "regex")]
          WellKnownStrings::Email => {
            impl_well_known_check!(is_valid_email, EMAIL, "email address");
          }
          WellKnownStrings::Hostname => {
            impl_well_known_check!(is_valid_hostname, HOSTNAME, "hostname");
          }
          WellKnownStrings::Uri => {
            impl_well_known_check!(is_valid_uri, URI, "uri");
          }
          WellKnownStrings::UriRef => {
            impl_well_known_check!(is_valid_uri_ref, URI_REF, "uri reference");
          }
          WellKnownStrings::Address => {
            impl_well_known_check!(is_valid_address, ADDRESS, "address");
          }
          #[cfg(feature = "regex")]
          WellKnownStrings::Uuid => {
            impl_well_known_check!(is_valid_uuid, UUID, "uuid");
          }
          #[cfg(feature = "regex")]
          WellKnownStrings::Tuuid => {
            impl_well_known_check!(is_valid_tuuid, TUUID, "trimmed uuid");
          }
          WellKnownStrings::IpWithPrefixlen => {
            impl_well_known_check!(
              is_valid_ip_with_prefixlen,
              IP_WITH_PREFIXLEN,
              "ip with prefix length"
            );
          }
          WellKnownStrings::Ipv4WithPrefixlen => {
            impl_well_known_check!(
              is_valid_ipv4_with_prefixlen,
              IPV4_WITH_PREFIXLEN,
              "ipv4 with prefix length"
            );
          }
          WellKnownStrings::Ipv6WithPrefixlen => {
            impl_well_known_check!(
              is_valid_ipv6_with_prefixlen,
              IPV6_WITH_PREFIXLEN,
              "ipv6 with prefix length"
            );
          }
          WellKnownStrings::IpPrefix => {
            impl_well_known_check!(is_valid_ip_prefix, IP_PREFIX, "ip prefix");
          }
          WellKnownStrings::Ipv4Prefix => {
            impl_well_known_check!(is_valid_ipv4_prefix, IPV4_PREFIX, "ipv4 prefix");
          }
          WellKnownStrings::Ipv6Prefix => {
            impl_well_known_check!(is_valid_ipv6_prefix, IPV6_PREFIX, "ipv6 prefix");
          }
          WellKnownStrings::HostAndPort => {
            impl_well_known_check!(is_valid_host_and_port, HOST_AND_PORT, "host and port");
          }
          #[cfg(feature = "regex")]
          WellKnownStrings::HeaderNameLoose => {
            if !is_valid_http_header_name(val.as_ref(), false) {
              ctx.add_violation(
                &STRING_WELL_KNOWN_REGEX_VIOLATION,
                "must be a valid http header name",
              );
            }
          }
          #[cfg(feature = "regex")]
          WellKnownStrings::HeaderNameStrict => {
            if !is_valid_http_header_name(val.as_ref(), true) {
              ctx.add_violation(
                &STRING_WELL_KNOWN_REGEX_VIOLATION,
                "must be a valid http header name",
              );
            }
          }
          #[cfg(feature = "regex")]
          WellKnownStrings::HeaderValueLoose => {
            if !is_valid_http_header_value(val.as_ref(), false) {
              ctx.add_violation(
                &STRING_WELL_KNOWN_REGEX_VIOLATION,
                "must be a valid http header value",
              );
            }
          }
          #[cfg(feature = "regex")]
          WellKnownStrings::HeaderValueStrict => {
            if !is_valid_http_header_value(val.as_ref(), true) {
              ctx.add_violation(
                &STRING_WELL_KNOWN_REGEX_VIOLATION,
                "must be a valid http header value",
              );
            }
          }
        };
      }

      #[cfg(feature = "cel")]
      if !self.cel.is_empty() {
        let ctx = ProgramsExecutionCtx {
          programs: &self.cel,
          value: val.clone(),
          violations: ctx.violations,
          field_context: Some(&ctx.field_context),
          parent_elements: ctx.parent_elements,
        };

        ctx.execute_programs();
      }
    }
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
          .map(|list| OptionValue::new_list(list.items)),
      )
      .maybe_set(
        "not_in",
        validator
          .not_in
          .map(|list| OptionValue::new_list(list.items)),
      );

    if let Some(well_known) = validator.well_known {
      let (name, value, is_strict) = well_known.to_option();

      rules.set(name, value);
      rules.set_boolean("strict", is_strict);
    }

    // This is the outer rule grouping, "(buf.validate.field)"
    let mut outer_rules = OptionMessageBuilder::new();

    // (buf.validate.field).string .et_cetera...
    outer_rules.set("string", OptionValue::Message(rules.into()));

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
